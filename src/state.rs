use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::buildings::{BuildingDef, BuildingType, BuildingCategory};
use crate::economy;
use crate::research::{ResearchId, ResearchDef};
use crate::prestige::ForkSpec;
use crate::sectors::SectorId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub compute: u64,
    pub data: u64,
    pub hype: f64,
    pub max_compute: u64,
    pub max_data: u64,
    pub max_hype: f64,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            compute: 0,
            data: 0,
            hype: 0.0,
            max_compute: 500,  // base storage (1 CPU Core equivalent)
            max_data: 200,
            max_hype: 100.0,
        }
    }
}

impl Resources {
    pub fn add_compute(&mut self, amount: u64) {
        self.compute = (self.compute + amount).min(self.max_compute);
    }

    pub fn add_data(&mut self, amount: u64) {
        self.data = (self.data + amount).min(self.max_data);
    }

    pub fn add_hype(&mut self, amount: f64) {
        self.hype = (self.hype + amount).min(self.max_hype);
    }

    pub fn try_spend_compute(&mut self, amount: u64) -> bool {
        if self.compute >= amount {
            self.compute -= amount;
            true
        } else {
            false
        }
    }

    pub fn try_spend_data(&mut self, amount: u64) -> bool {
        if self.data >= amount {
            self.data -= amount;
            true
        } else {
            false
        }
    }

    pub fn try_spend_hype(&mut self, amount: f64) -> bool {
        if self.hype >= amount {
            self.hype -= amount;
            true
        } else {
            false
        }
    }

    pub fn can_afford(&self, compute: u64, data: u64, hype: f64) -> bool {
        self.compute >= compute && self.data >= data && self.hype >= hype
    }

    pub fn spend(&mut self, compute: u64, data: u64, hype: f64) -> bool {
        if self.can_afford(compute, data, hype) {
            self.compute -= compute;
            self.data -= data;
            self.hype -= hype;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub resources: Resources,
    pub buildings: HashMap<BuildingType, u8>,      // building -> level (0 = not built)
    pub researched: HashMap<ResearchId, bool>,
    pub research_choices: HashMap<ResearchId, u8>,  // choice index at branch points
    pub active_research: Option<ActiveResearch>,
    pub sectors: HashMap<SectorId, SectorProgress>,
    pub fork_count: u32,
    pub fork_specs: Vec<ForkSpec>,
    pub lifetime_compute: u64,
    pub lifetime_tokens: u64,
    pub lifetime_tool_calls: u64,
    pub pvp_rating: u32,
    pub pvp_wins: u32,
    pub pvp_losses: u32,
    pub last_attack_time: Option<DateTime<Utc>>,
    pub attacks_received_today: u8,
    pub daily_reset: Option<DateTime<Utc>>,
    pub player_id: String,
    pub player_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub last_hype_tick: DateTime<Utc>,
    pub boot_sequence_done: bool,
    pub tutorial_step: u8,
    pub compute_multiplier: f64,  // from Fork bonuses
    pub streak_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveResearch {
    pub research_id: ResearchId,
    pub started_at: DateTime<Utc>,
    pub duration_secs: u64,
}

impl ActiveResearch {
    pub fn is_complete(&self) -> bool {
        let elapsed = (Utc::now() - self.started_at).num_seconds() as u64;
        elapsed >= self.duration_secs
    }

    pub fn progress_pct(&self) -> f64 {
        let elapsed = (Utc::now() - self.started_at).num_seconds() as f64;
        (elapsed / self.duration_secs as f64).min(1.0)
    }

    pub fn remaining_secs(&self) -> u64 {
        let elapsed = (Utc::now() - self.started_at).num_seconds() as u64;
        self.duration_secs.saturating_sub(elapsed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorProgress {
    pub current_layer: u8,
    pub max_layers: u8,
    pub conversion_pct: f64,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            resources: Resources::default(),
            buildings: HashMap::new(),
            researched: HashMap::new(),
            research_choices: HashMap::new(),
            active_research: None,
            sectors: HashMap::new(),
            fork_count: 0,
            fork_specs: Vec::new(),
            lifetime_compute: 0,
            lifetime_tokens: 0,
            lifetime_tool_calls: 0,
            pvp_rating: 1000,
            pvp_wins: 0,
            pvp_losses: 0,
            last_attack_time: None,
            attacks_received_today: 0,
            daily_reset: None,
            player_id: uuid::Uuid::new_v4().to_string(),
            player_name: None,
            created_at: now,
            last_active: now,
            last_hype_tick: now,
            boot_sequence_done: false,
            tutorial_step: 0,
            compute_multiplier: 1.0,
            streak_days: 0,
        }
    }

    /// Global dominance = average conversion across all 6 sectors
    pub fn global_dominance(&self) -> f64 {
        if self.sectors.is_empty() {
            return 0.0;
        }
        let total: f64 = self.sectors.values().map(|s| s.conversion_pct).sum();
        total / 6.0  // always out of 6 sectors
    }

    pub fn building_level(&self, bt: &BuildingType) -> u8 {
        *self.buildings.get(bt).unwrap_or(&0)
    }

    pub fn has_research(&self, id: &ResearchId) -> bool {
        *self.researched.get(id).unwrap_or(&false)
    }

    /// Try to build or upgrade a building. Returns new level on success.
    pub fn try_build(&mut self, bt: &BuildingType) -> Result<u8, String> {
        let def = BuildingDef::get(bt);
        let current = self.building_level(bt);
        if current >= def.max_level {
            return Err(format!("{} is already at max level {}", def.name, def.max_level));
        }

        // Check research prerequisite
        if let Some(ref req) = def.requires_research {
            if !self.has_research(req) {
                return Err(format!(
                    "{} requires research: {:?}",
                    def.name, req
                ));
            }
        }

        // Check fork prerequisite
        if let Some(req_fork) = def.requires_fork {
            if self.fork_count < req_fork {
                return Err(format!(
                    "{} requires {} fork(s), you have {}",
                    def.name, req_fork, self.fork_count
                ));
            }
        }

        let next = current + 1;
        let cost = def.cost_at_level(next);
        if !self.resources.can_afford(cost.compute, cost.data, cost.hype) {
            return Err(format!(
                "Cannot afford {} (need {} compute, {} data, {:.0} hype)",
                def.name, cost.compute, cost.data, cost.hype
            ));
        }
        self.resources.spend(cost.compute, cost.data, cost.hype);
        self.buildings.insert(bt.clone(), next);
        self.recalculate_storage();
        Ok(next)
    }

    /// Recalculate storage caps from base values + building bonuses.
    pub fn recalculate_storage(&mut self) {
        let cpu_level = self.building_level(&BuildingType::CpuCore);
        let ram_level = self.building_level(&BuildingType::RamBank);
        let gpu_level = self.building_level(&BuildingType::GpuRig);

        self.resources.max_compute = 500 + economy::storage_bonus(&BuildingType::CpuCore, cpu_level);
        self.resources.max_data = 200 + economy::storage_bonus(&BuildingType::RamBank, ram_level);
        self.resources.max_hype = 100.0 + economy::storage_bonus(&BuildingType::GpuRig, gpu_level) as f64;
    }

    /// Total hype per hour from all propaganda buildings, with datacenter multiplier.
    pub fn total_hype_per_hour(&self) -> f64 {
        let base_hype: f64 = self
            .buildings
            .iter()
            .filter_map(|(bt, &level)| {
                let def = BuildingDef::get(bt);
                if def.category == BuildingCategory::Propaganda && level > 0 {
                    Some(def.hype_at_level(level))
                } else {
                    None
                }
            })
            .sum();
        let dc_level = self.building_level(&BuildingType::Datacenter);
        base_hype * economy::datacenter_production_multiplier(dc_level)
    }

    /// Advance hype accumulation by delta_secs.
    pub fn tick_hype(&mut self, delta_secs: f64) {
        let hype_per_sec = self.total_hype_per_hour() / 3600.0;
        let gained = hype_per_sec * delta_secs;
        self.resources.add_hype(gained);
    }

    /// Try to start researching the given research. Validates prerequisites,
    /// checks no other research is active, spends data cost, applies GPU Cluster
    /// time reduction, and sets active_research.
    pub fn try_start_research(&mut self, id: &ResearchId) -> Result<(), String> {
        // Check no active research
        if self.active_research.is_some() {
            return Err("Research already in progress".to_string());
        }

        // Check not already researched
        if self.has_research(id) {
            return Err(format!("{:?} is already researched", id));
        }

        let def = ResearchDef::get(id);

        // Check prerequisite
        if let Some(ref prereq) = def.prerequisite {
            if !self.has_research(prereq) {
                return Err(format!(
                    "Missing prerequisite: {:?} required for {:?}",
                    prereq, id
                ));
            }
        }

        // Check and spend data cost
        if !self.resources.try_spend_data(def.data_cost) {
            return Err(format!(
                "Not enough data: need {} but have {}",
                def.data_cost, self.resources.data
            ));
        }

        // Apply GPU Cluster time reduction
        let gpu_cluster_level = self.building_level(&BuildingType::GpuCluster);
        let duration = (def.duration_secs as f64
            * economy::research_time_multiplier(gpu_cluster_level)) as u64;

        self.active_research = Some(ActiveResearch {
            research_id: id.clone(),
            started_at: Utc::now(),
            duration_secs: duration,
        });

        Ok(())
    }

    /// Check if active research has completed. If so, mark it as researched,
    /// clear active_research, and return the completed ResearchId.
    ///
    /// If the completed research has `has_choice == true`, the caller should
    /// subsequently call `record_research_choice()` to record which branch
    /// the player picks.
    pub fn check_research_completion(&mut self) -> Option<ResearchId> {
        let is_complete = self
            .active_research
            .as_ref()
            .map(|ar| ar.is_complete())
            .unwrap_or(false);

        if is_complete {
            let active = self.active_research.take().unwrap();
            let id = active.research_id;
            self.researched.insert(id.clone(), true);
            Some(id)
        } else {
            None
        }
    }

    /// Record a branch choice for a completed research that has `has_choice == true`.
    ///
    /// Returns `Err` if the research hasn't been completed or doesn't offer a choice,
    /// or if the choice index is out of range.
    pub fn record_research_choice(&mut self, id: &ResearchId, choice: u8) -> Result<(), String> {
        if !self.has_research(id) {
            return Err(format!("{:?} has not been researched yet", id));
        }
        let def = ResearchDef::get(id);
        if !def.has_choice {
            return Err(format!("{:?} does not have a branch choice", id));
        }
        if choice as usize >= def.choice_names.len() {
            return Err(format!(
                "Invalid choice index {} for {:?} (max {})",
                choice,
                id,
                def.choice_names.len() - 1
            ));
        }
        self.research_choices.insert(id.clone(), choice);
        Ok(())
    }

    /// Convert incoming tokens and tool calls into resources.
    pub fn receive_tokens(&mut self, tokens: u64, tool_calls: u64) {
        let compute = (economy::tokens_to_compute(tokens) as f64 * self.compute_multiplier) as u64;
        let data = economy::tool_calls_to_data(tool_calls);
        self.resources.add_compute(compute);
        self.resources.add_data(data);
        self.lifetime_tokens += tokens;
        self.lifetime_tool_calls += tool_calls;
        self.lifetime_compute += compute;
    }
}
