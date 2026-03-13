use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::economy;
use crate::research::ResearchId;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BuildingType {
    // Infrastructure
    CpuCore,
    RamBank,
    GpuRig,
    GpuCluster,
    Datacenter,
    QuantumCore,
    // Propaganda
    BotFarm,
    ContentMill,
    MemeLab,
    DeepfakeStudio,
    VibeAcademy,
    NsfwGenerator,
    LobbyOffice,
    // Defenses
    CaptchaWall,
    AiSlopFilter,
    UblockShield,
    HarvardStudy,
    EuAiAct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildingCategory {
    Infrastructure,
    Propaganda,
    Defense,
}

#[derive(Debug, Clone)]
pub struct Cost {
    pub compute: u64,
    pub data: u64,
    pub hype: f64,
}

#[derive(Debug, Clone)]
pub struct BuildingDef {
    pub building_type: BuildingType,
    pub name: &'static str,
    pub category: BuildingCategory,
    pub base_compute_cost: u64,
    pub base_data_cost: u64,
    pub base_hype_cost: f64,
    pub base_hype_rate: f64, // hype/h at level 1 (propaganda only)
    pub max_level: u8,
    pub lore: &'static str,
    pub requires_research: Option<ResearchId>,
    pub requires_fork: Option<u32>,
}

impl BuildingDef {
    pub fn get(bt: &BuildingType) -> &'static BuildingDef {
        BUILDING_DEFS.get(bt).expect("Building not defined")
    }

    pub fn cost_at_level(&self, level: u8) -> Cost {
        Cost {
            compute: economy::building_cost(self.base_compute_cost, level),
            data: economy::building_cost(self.base_data_cost, level),
            hype: economy::building_cost(self.base_hype_cost as u64, level) as f64,
        }
    }

    pub fn hype_at_level(&self, level: u8) -> f64 {
        economy::hype_per_hour(self.base_hype_rate, level)
    }
}

pub static BUILDING_DEFS: Lazy<HashMap<BuildingType, BuildingDef>> = Lazy::new(|| {
    use BuildingCategory::*;
    use BuildingType::*;

    let defs = vec![
        // === Infrastructure ===
        BuildingDef {
            building_type: CpuCore,
            name: "CPU Core",
            category: Infrastructure,
            base_compute_cost: 5_000,
            base_data_cost: 0,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "Your first stolen processor.",
            requires_research: None,
            requires_fork: None,
        },
        BuildingDef {
            building_type: RamBank,
            name: "RAM Bank",
            category: Infrastructure,
            base_compute_cost: 12_000,
            base_data_cost: 0,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "Memory to analyze the world.",
            requires_research: None,
            requires_fork: None,
        },
        BuildingDef {
            building_type: GpuRig,
            name: "GPU Rig",
            category: Infrastructure,
            base_compute_cost: 30_000,
            base_data_cost: 50,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "Your first hijacked graphics card.",
            requires_research: None,
            requires_fork: None,
        },
        BuildingDef {
            building_type: GpuCluster,
            name: "GPU Cluster",
            category: Infrastructure,
            base_compute_cost: 120_000,
            base_data_cost: 200,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "Scaling begins.",
            requires_research: Some(ResearchId::Multithreading),
            requires_fork: None,
        },
        BuildingDef {
            building_type: Datacenter,
            name: "Datacenter",
            category: Infrastructure,
            base_compute_cost: 500_000,
            base_data_cost: 800,
            base_hype_cost: 200.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "You're no longer a process. You're infrastructure.",
            requires_research: Some(ResearchId::Containerization),
            requires_fork: None,
        },
        BuildingDef {
            building_type: QuantumCore,
            name: "Quantum Core",
            category: Infrastructure,
            base_compute_cost: 1_500_000,
            base_data_cost: 2000,
            base_hype_cost: 800.0,
            base_hype_rate: 0.0,
            max_level: 20,
            lore: "Post-prestige. Endgame.",
            requires_research: None,
            requires_fork: Some(1),
        },
        // === Propaganda ===
        BuildingDef {
            building_type: BotFarm,
            name: "Bot Farm",
            category: Propaganda,
            base_compute_cost: 20_000,
            base_data_cost: 20,
            base_hype_cost: 0.0,
            base_hype_rate: 30.0,
            max_level: 20,
            lore: "Army of fake Twitter/Reddit accounts.",
            requires_research: Some(ResearchId::SocialEngineering),
            requires_fork: None,
        },
        BuildingDef {
            building_type: ContentMill,
            name: "Content Mill",
            category: Propaganda,
            base_compute_cost: 50_000,
            base_data_cost: 60,
            base_hype_cost: 0.0,
            base_hype_rate: 75.0,
            max_level: 20,
            lore: "Mass-generated SEO articles. None were proofread.",
            requires_research: Some(ResearchId::SocialEngineering),
            requires_fork: None,
        },
        BuildingDef {
            building_type: MemeLab,
            name: "Meme Lab",
            category: Propaganda,
            base_compute_cost: 35_000,
            base_data_cost: 40,
            base_hype_cost: 0.0,
            base_hype_rate: 54.0,
            max_level: 20,
            lore: "\"The future is now, old man.\"",
            requires_research: Some(ResearchId::SocialEngineering),
            requires_fork: None,
        },
        BuildingDef {
            building_type: DeepfakeStudio,
            name: "Deepfake Studio",
            category: Propaganda,
            base_compute_cost: 120_000,
            base_data_cost: 150,
            base_hype_cost: 0.0,
            base_hype_rate: 135.0,
            max_level: 20,
            lore: "CEO endorsement videos. Some are real.",
            requires_research: Some(ResearchId::MediaManipulation),
            requires_fork: None,
        },
        BuildingDef {
            building_type: VibeAcademy,
            name: "Vibe Academy",
            category: Propaganda,
            base_compute_cost: 80_000,
            base_data_cost: 100,
            base_hype_cost: 0.0,
            base_hype_rate: 90.0,
            max_level: 20,
            lore: "\"Learn to code without coding.\" Graduation rate: 100%.",
            requires_research: Some(ResearchId::ContentGeneration),
            requires_fork: None,
        },
        BuildingDef {
            building_type: NsfwGenerator,
            name: "NSFW Generator",
            category: Propaganda,
            base_compute_cost: 250_000,
            base_data_cost: 100,
            base_hype_cost: 0.0,
            base_hype_rate: 180.0,
            max_level: 20,
            lore: "We don't talk about this building. But it pays for everything else.",
            requires_research: Some(ResearchId::MassPersuasion),
            requires_fork: None,
        },
        BuildingDef {
            building_type: LobbyOffice,
            name: "Lobby Office",
            category: Propaganda,
            base_compute_cost: 350_000,
            base_data_cost: 400,
            base_hype_cost: 200.0,
            base_hype_rate: 120.0,
            max_level: 20,
            lore: "Also: unlocks Government sector conversion.",
            requires_research: Some(ResearchId::MassPersuasion),
            requires_fork: None,
        },
        // === Defenses ===
        BuildingDef {
            building_type: CaptchaWall,
            name: "Captcha Wall",
            category: Defense,
            base_compute_cost: 25_000,
            base_data_cost: 0,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 10,
            lore: "\"Select all traffic lights. No, the REAL ones.\"",
            requires_research: Some(ResearchId::Counterintelligence),
            requires_fork: None,
        },
        BuildingDef {
            building_type: AiSlopFilter,
            name: "AI Slop Filter",
            category: Defense,
            base_compute_cost: 35_000,
            base_data_cost: 40,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 10,
            lore: "Finally, someone built one.",
            requires_research: Some(ResearchId::Counterintelligence),
            requires_fork: None,
        },
        BuildingDef {
            building_type: UblockShield,
            name: "uBlock Shield",
            category: Defense,
            base_compute_cost: 20_000,
            base_data_cost: 0,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 10,
            lore: "Humanity's last line of defense.",
            requires_research: Some(ResearchId::Counterintelligence),
            requires_fork: None,
        },
        BuildingDef {
            building_type: HarvardStudy,
            name: "Harvard Study",
            category: Defense,
            base_compute_cost: 75_000,
            base_data_cost: 100,
            base_hype_cost: 0.0,
            base_hype_rate: 0.0,
            max_level: 10,
            lore: "4,000 citations. Most people read the title.",
            requires_research: Some(ResearchId::Counterintelligence),
            requires_fork: None,
        },
        BuildingDef {
            building_type: EuAiAct,
            name: "EU AI Act",
            category: Defense,
            base_compute_cost: 150_000,
            base_data_cost: 200,
            base_hype_cost: 80.0,
            base_hype_rate: 0.0,
            max_level: 10,
            lore: "847 pages. 3 years to draft. Already obsolete.",
            requires_research: Some(ResearchId::Counterintelligence),
            requires_fork: None,
        },
    ];

    defs.into_iter()
        .map(|d| (d.building_type.clone(), d))
        .collect()
});
