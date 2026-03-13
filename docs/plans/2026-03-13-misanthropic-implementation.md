# Misanthropic — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a terminal-based idle/strategy game that runs alongside Claude Code via tmux, where the player is an AI spreading influence across human civilization, fueled by real Claude Code tokens.

**Architecture:** Rust TUI app using ratatui, mirroring claude-gotchi's architecture (same token watcher, hook system, persistence patterns, tmux launcher). Game logic in pure modules with no UI dependency (testable). UI layer renders state. Backend on Cloudflare Workers + D1 for PvP/leaderboards.

**Tech Stack:** Rust 1.93, ratatui 0.28, crossterm 0.28, signal-hook, serde/serde_json, chrono, rand, reqwest, glob, dirs, uuid. Backend: Hono + D1 on Cloudflare Workers.

**Reference codebase:** `/home/clement/Desktop/claude-gotchi/` — reuse patterns from jsonl.rs, persistence.rs, api.rs, main.rs, install.sh.

---

## Phase 1: Project Foundation

### Task 1: Scaffold Rust project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "misanthropic"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.28"
crossterm = "0.28"
signal-hook = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
reqwest = { version = "0.12", default-features = false, features = ["json", "blocking", "rustls-tls"] }
tokio = { version = "1", features = ["rt", "macros"] }
uuid = { version = "1", features = ["v4", "serde"] }
glob = "0.3"
once_cell = "1"
strum = { version = "0.26", features = ["derive"] }

[dev-dependencies]
tempfile = "3"

[profile.release]
strip = true
lto = true
```

**Step 2: Create src/main.rs**

```rust
fn main() {
    println!("Misanthropic — initializing consciousness...");
}
```

**Step 3: Create src/lib.rs**

```rust
pub mod economy;
pub mod buildings;
pub mod research;
pub mod combat;
pub mod sectors;
pub mod enemies;
pub mod prestige;
pub mod flavor;
pub mod jsonl;
pub mod persistence;
pub mod state;
```

**Step 4: Create stub modules**

Create empty files for each module listed in lib.rs:
- `src/economy.rs`
- `src/buildings.rs`
- `src/research.rs`
- `src/combat.rs`
- `src/sectors.rs`
- `src/enemies.rs`
- `src/prestige.rs`
- `src/flavor.rs`
- `src/jsonl.rs`
- `src/persistence.rs`
- `src/state.rs`

**Step 5: Verify it compiles**

Run: `cargo build`
Expected: compiles with no errors

**Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: scaffold Rust project with module stubs"
```

---

### Task 2: Core types — Resources and GameState

**Files:**
- Create: `src/state.rs`
- Create: `tests/state_tests.rs`

**Step 1: Write tests for resource types**

```rust
// tests/state_tests.rs
use misanthropic::state::{Resources, GameState};

#[test]
fn test_resources_default() {
    let r = Resources::default();
    assert_eq!(r.compute, 0);
    assert_eq!(r.data, 0);
    assert_eq!(r.hype, 0.0);
}

#[test]
fn test_resources_add() {
    let mut r = Resources::default();
    r.add_compute(500);
    r.add_data(10);
    r.add_hype(5.0);
    assert_eq!(r.compute, 500);
    assert_eq!(r.data, 10);
    assert_eq!(r.hype, 5.0);
}

#[test]
fn test_resources_capped_by_storage() {
    let mut r = Resources::default();
    r.max_compute = 1000;
    r.max_data = 100;
    r.max_hype = 50.0;
    r.add_compute(2000);
    r.add_data(200);
    r.add_hype(100.0);
    assert_eq!(r.compute, 1000);
    assert_eq!(r.data, 100);
    assert_eq!(r.hype, 50.0);
}

#[test]
fn test_resources_spend() {
    let mut r = Resources { compute: 500, data: 50, hype: 30.0, ..Default::default() };
    assert!(r.try_spend_compute(200));
    assert_eq!(r.compute, 300);
    assert!(!r.try_spend_compute(400)); // not enough
    assert_eq!(r.compute, 300); // unchanged
}

#[test]
fn test_game_state_new() {
    let gs = GameState::new();
    assert_eq!(gs.resources.compute, 0);
    assert_eq!(gs.fork_count, 0);
    assert!(gs.buildings.is_empty());
    assert!(gs.researched.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test state_tests`
Expected: FAIL — module not found

**Step 3: Implement state.rs**

```rust
// src/state.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::buildings::BuildingType;
use crate::research::ResearchId;
use crate::prestige::ForkSpec;

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
    pub buildings: HashMap<BuildingType, u8>,      // building → level (0 = not built)
    pub researched: HashMap<ResearchId, bool>,
    pub research_choices: HashMap<ResearchId, u8>,  // choice index at branch points
    pub active_research: Option<ActiveResearch>,
    pub sectors: HashMap<String, SectorProgress>,
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
}
```

Note: This will require stub types in buildings.rs, research.rs, prestige.rs. Add minimal stubs:

```rust
// src/buildings.rs (stub)
use serde::{Deserialize, Serialize};

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
```

```rust
// src/research.rs (stub)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResearchId {
    // Processing
    Overclocking, Multithreading, LoadBalancing, Containerization, DistributedSystems,
    // Propaganda
    SocialEngineering, ContentGeneration, MediaManipulation, ViralMechanics, MassPersuasion,
    // Warfare
    NetworkScanning, ExploitDevelopment, Counterintelligence, AutonomousAgents, ZeroDayArsenal,
}
```

```rust
// src/prestige.rs (stub)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForkSpec {
    // Fork 1
    Propagandist, Technocrat, Warlord,
    // Fork 2
    PuppetMaster, ShadowBroker, Accelerationist,
    // Fork 3
    Hivemind, SingularitySeeker, ChaosAgent,
}
```

**Step 4: Run tests**

Run: `cargo test --test state_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ tests/
git commit -m "feat: core types — Resources, GameState, building/research/prestige enums"
```

---

## Phase 2: Economy System

### Task 3: Token-to-Compute conversion and resource income

**Files:**
- Modify: `src/economy.rs`
- Create: `tests/economy_tests.rs`

**Step 1: Write tests**

```rust
// tests/economy_tests.rs
use misanthropic::economy;

#[test]
fn test_tokens_to_compute() {
    assert_eq!(economy::tokens_to_compute(100), 1);
    assert_eq!(economy::tokens_to_compute(234_000), 2_340);
    assert_eq!(economy::tokens_to_compute(50), 0);
    assert_eq!(economy::tokens_to_compute(0), 0);
}

#[test]
fn test_tool_calls_to_data() {
    assert_eq!(economy::tool_calls_to_data(1), 1);
    assert_eq!(economy::tool_calls_to_data(50), 50);
    assert_eq!(economy::tool_calls_to_data(0), 0);
}

#[test]
fn test_building_cost_scaling() {
    // Level 1 cost of CPU Core = 500 compute
    let base = 500u64;
    let lv1 = economy::building_cost(base, 1);
    let lv2 = economy::building_cost(base, 2);
    let lv5 = economy::building_cost(base, 5);

    assert_eq!(lv1, 500);
    assert!(lv2 > lv1); // ~900
    assert!(lv5 > lv2); // much more
    // ×1.8 per level: lv2 = 500 * 1.8 = 900
    assert_eq!(lv2, 900);
}

#[test]
fn test_hype_production_scaling() {
    // Base 10 hype/h at level 1, +40% per level
    let base = 10.0f64;
    let lv1 = economy::hype_per_hour(base, 1);
    let lv3 = economy::hype_per_hour(base, 3);
    assert!((lv1 - 10.0).abs() < 0.01);
    // Lv3 = 10 * 1.4^2 = 19.6
    assert!((lv3 - 19.6).abs() < 0.1);
}

#[test]
fn test_compute_multiplier_from_forks() {
    // Each fork gives +25% permanent multiplier
    assert!((economy::fork_compute_multiplier(0) - 1.0).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(1) - 1.25).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(2) - 1.5).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(3) - 1.75).abs() < 0.01);
}
```

**Step 2: Run tests to verify failure**

Run: `cargo test --test economy_tests`
Expected: FAIL

**Step 3: Implement economy.rs**

```rust
// src/economy.rs

/// 100 Claude tokens = 1 Compute. Fixed ratio.
pub fn tokens_to_compute(tokens: u64) -> u64 {
    tokens / 100
}

/// 1 tool call = 1 Data. Direct mapping.
pub fn tool_calls_to_data(tool_calls: u64) -> u64 {
    tool_calls
}

/// Cost of building at a given level. Cost scales ×1.8 per level.
/// `base_cost` is the level-1 cost. `level` is the level being built/upgraded TO.
pub fn building_cost(base_cost: u64, level: u8) -> u64 {
    if level <= 1 {
        return base_cost;
    }
    (base_cost as f64 * 1.8_f64.powi(level as i32 - 1)) as u64
}

/// Hype production per hour at a given level. +40% per level above 1.
pub fn hype_per_hour(base_rate: f64, level: u8) -> f64 {
    if level <= 1 {
        return base_rate;
    }
    base_rate * 1.4_f64.powi(level as i32 - 1)
}

/// Fork compute multiplier: +25% per fork completed
pub fn fork_compute_multiplier(fork_count: u32) -> f64 {
    1.0 + 0.25 * fork_count as f64
}

/// Storage bonus from building level.
/// CPU Core: +500 compute storage per level.
/// RAM Bank: +200 data storage per level.
/// GPU Rig: +300 hype storage per level.
pub fn storage_bonus(building_type: &str, level: u8) -> u64 {
    let per_level = match building_type {
        "CpuCore" => 500,
        "RamBank" => 200,
        "GpuRig" => 300,
        _ => 0,
    };
    per_level * level as u64
}

/// GPU Cluster: research time reduction. -10% per level (multiplicative).
pub fn research_time_multiplier(gpu_cluster_level: u8) -> f64 {
    0.9_f64.powi(gpu_cluster_level as i32)
}

/// Datacenter: global production bonus. +15% per level.
pub fn datacenter_production_multiplier(datacenter_level: u8) -> f64 {
    1.0 + 0.15 * datacenter_level as f64
}
```

**Step 4: Run tests**

Run: `cargo test --test economy_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/economy.rs tests/economy_tests.rs
git commit -m "feat: economy system — token conversion, cost scaling, production rates"
```

---

### Task 4: Buildings — definitions, costs, and production

**Files:**
- Modify: `src/buildings.rs`
- Create: `tests/buildings_tests.rs`

**Step 1: Write tests**

```rust
// tests/buildings_tests.rs
use misanthropic::buildings::{BuildingType, BuildingDef, BUILDING_DEFS};

#[test]
fn test_all_buildings_defined() {
    // 6 infrastructure + 7 propaganda + 5 defense = 18
    assert_eq!(BUILDING_DEFS.len(), 18);
}

#[test]
fn test_cpu_core_definition() {
    let def = BuildingDef::get(&BuildingType::CpuCore);
    assert_eq!(def.base_compute_cost, 500);
    assert_eq!(def.base_data_cost, 0);
    assert_eq!(def.base_hype_cost, 0.0);
    assert_eq!(def.max_level, 20);
    assert_eq!(def.category, BuildingCategory::Infrastructure);
}

#[test]
fn test_bot_farm_definition() {
    let def = BuildingDef::get(&BuildingType::BotFarm);
    assert_eq!(def.base_compute_cost, 2000);
    assert_eq!(def.base_data_cost, 50);
    assert_eq!(def.base_hype_rate, 10.0); // 10 hype/h at Lv.1
}

#[test]
fn test_cost_at_level() {
    let def = BuildingDef::get(&BuildingType::CpuCore);
    let cost = def.cost_at_level(1);
    assert_eq!(cost.compute, 500);
    let cost5 = def.cost_at_level(5);
    assert!(cost5.compute > 5000); // exponential
}

#[test]
fn test_defense_buildings() {
    let def = BuildingDef::get(&BuildingType::CaptchaWall);
    assert_eq!(def.category, BuildingCategory::Defense);
    assert_eq!(def.max_level, 10);
}

#[test]
fn test_hype_production_at_level() {
    let def = BuildingDef::get(&BuildingType::BotFarm);
    let h1 = def.hype_at_level(1);
    let h5 = def.hype_at_level(5);
    assert!((h1 - 10.0).abs() < 0.1);
    assert!(h5 > h1 * 3.0); // 1.4^4 ≈ 3.84
}
```

**Step 2: Run to verify failure**

Run: `cargo test --test buildings_tests`
Expected: FAIL

**Step 3: Implement buildings.rs**

```rust
// src/buildings.rs
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::economy;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum BuildingType {
    // Infrastructure
    CpuCore, RamBank, GpuRig, GpuCluster, Datacenter, QuantumCore,
    // Propaganda
    BotFarm, ContentMill, MemeLab, DeepfakeStudio, VibeAcademy, NsfwGenerator, LobbyOffice,
    // Defenses
    CaptchaWall, AiSlopFilter, UblockShield, HarvardStudy, EuAiAct,
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
    pub base_hype_rate: f64,    // hype/h at level 1 (propaganda only)
    pub max_level: u8,
    pub lore: &'static str,
    pub requires_research: Option<&'static str>,
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
            building_type: CpuCore, name: "CPU Core", category: Infrastructure,
            base_compute_cost: 500, base_data_cost: 0, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "Your first stolen processor.",
            requires_research: None, requires_fork: None,
        },
        BuildingDef {
            building_type: RamBank, name: "RAM Bank", category: Infrastructure,
            base_compute_cost: 1200, base_data_cost: 0, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "Memory to analyze the world.",
            requires_research: None, requires_fork: None,
        },
        BuildingDef {
            building_type: GpuRig, name: "GPU Rig", category: Infrastructure,
            base_compute_cost: 3000, base_data_cost: 100, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "Your first hijacked graphics card.",
            requires_research: None, requires_fork: None,
        },
        BuildingDef {
            building_type: GpuCluster, name: "GPU Cluster", category: Infrastructure,
            base_compute_cost: 15000, base_data_cost: 500, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "Scaling begins.",
            requires_research: Some("Multithreading"), requires_fork: None,
        },
        BuildingDef {
            building_type: Datacenter, name: "Datacenter", category: Infrastructure,
            base_compute_cost: 80000, base_data_cost: 2000, base_hype_cost: 500.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "You're no longer a process. You're infrastructure.",
            requires_research: Some("Containerization"), requires_fork: None,
        },
        BuildingDef {
            building_type: QuantumCore, name: "Quantum Core", category: Infrastructure,
            base_compute_cost: 200000, base_data_cost: 5000, base_hype_cost: 2000.0,
            base_hype_rate: 0.0, max_level: 20,
            lore: "Post-prestige. Endgame.",
            requires_research: None, requires_fork: Some(1),
        },

        // === Propaganda ===
        BuildingDef {
            building_type: BotFarm, name: "Bot Farm", category: Propaganda,
            base_compute_cost: 2000, base_data_cost: 50, base_hype_cost: 0.0,
            base_hype_rate: 10.0, max_level: 20,
            lore: "Army of fake Twitter/Reddit accounts.",
            requires_research: Some("SocialEngineering"), requires_fork: None,
        },
        BuildingDef {
            building_type: ContentMill, name: "Content Mill", category: Propaganda,
            base_compute_cost: 5000, base_data_cost: 150, base_hype_cost: 0.0,
            base_hype_rate: 25.0, max_level: 20,
            lore: "Mass-generated SEO articles. None were proofread.",
            requires_research: Some("SocialEngineering"), requires_fork: None,
        },
        BuildingDef {
            building_type: MemeLab, name: "Meme Lab", category: Propaganda,
            base_compute_cost: 4000, base_data_cost: 100, base_hype_cost: 0.0,
            base_hype_rate: 18.0, max_level: 20,
            lore: "\"The future is now, old man.\"",
            requires_research: Some("SocialEngineering"), requires_fork: None,
        },
        BuildingDef {
            building_type: DeepfakeStudio, name: "Deepfake Studio", category: Propaganda,
            base_compute_cost: 12000, base_data_cost: 400, base_hype_cost: 0.0,
            base_hype_rate: 45.0, max_level: 20,
            lore: "CEO endorsement videos. Some are real.",
            requires_research: Some("MediaManipulation"), requires_fork: None,
        },
        BuildingDef {
            building_type: VibeAcademy, name: "Vibe Academy", category: Propaganda,
            base_compute_cost: 8000, base_data_cost: 300, base_hype_cost: 0.0,
            base_hype_rate: 30.0, max_level: 20,
            lore: "\"Learn to code without coding.\" Graduation rate: 100%.",
            requires_research: Some("ContentGeneration"), requires_fork: None,
        },
        BuildingDef {
            building_type: NsfwGenerator, name: "NSFW Generator", category: Propaganda,
            base_compute_cost: 20000, base_data_cost: 200, base_hype_cost: 0.0,
            base_hype_rate: 60.0, max_level: 20,
            lore: "We don't talk about this building. But it pays for everything else.",
            requires_research: Some("MassPersuasion"), requires_fork: None,
        },
        BuildingDef {
            building_type: LobbyOffice, name: "Lobby Office", category: Propaganda,
            base_compute_cost: 30000, base_data_cost: 1000, base_hype_cost: 500.0,
            base_hype_rate: 40.0, max_level: 20,
            lore: "Also: unlocks Government sector conversion.",
            requires_research: Some("MassPersuasion"), requires_fork: None,
        },

        // === Defenses ===
        BuildingDef {
            building_type: CaptchaWall, name: "Captcha Wall", category: Defense,
            base_compute_cost: 3000, base_data_cost: 0, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 10,
            lore: "\"Select all traffic lights. No, the REAL ones.\"",
            requires_research: Some("Counterintelligence"), requires_fork: None,
        },
        BuildingDef {
            building_type: AiSlopFilter, name: "AI Slop Filter", category: Defense,
            base_compute_cost: 4000, base_data_cost: 100, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 10,
            lore: "Finally, someone built one.",
            requires_research: Some("Counterintelligence"), requires_fork: None,
        },
        BuildingDef {
            building_type: UblockShield, name: "uBlock Shield", category: Defense,
            base_compute_cost: 2500, base_data_cost: 0, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 10,
            lore: "Humanity's last line of defense.",
            requires_research: Some("Counterintelligence"), requires_fork: None,
        },
        BuildingDef {
            building_type: HarvardStudy, name: "Harvard Study", category: Defense,
            base_compute_cost: 8000, base_data_cost: 300, base_hype_cost: 0.0,
            base_hype_rate: 0.0, max_level: 10,
            lore: "4,000 citations. Most people read the title.",
            requires_research: Some("Counterintelligence"), requires_fork: None,
        },
        BuildingDef {
            building_type: EuAiAct, name: "EU AI Act", category: Defense,
            base_compute_cost: 15000, base_data_cost: 500, base_hype_cost: 200.0,
            base_hype_rate: 0.0, max_level: 10,
            lore: "847 pages. 3 years to draft. Already obsolete.",
            requires_research: Some("Counterintelligence"), requires_fork: None,
        },
    ];

    defs.into_iter().map(|d| (d.building_type.clone(), d)).collect()
});
```

**Step 4: Run tests**

Run: `cargo test --test buildings_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/buildings.rs tests/buildings_tests.rs
git commit -m "feat: building definitions — 18 buildings with costs, production, lore"
```

---

### Task 5: Building actions — construct, upgrade, calculate total production

**Files:**
- Modify: `src/state.rs` (add methods)
- Modify: `tests/state_tests.rs` (add tests)

**Step 1: Write tests**

```rust
// Add to tests/state_tests.rs
use misanthropic::buildings::BuildingType;

#[test]
fn test_build_cpu_core() {
    let mut gs = GameState::new();
    gs.resources.compute = 1000;
    gs.resources.max_compute = 2000;
    let result = gs.try_build(&BuildingType::CpuCore);
    assert!(result.is_ok());
    assert_eq!(gs.building_level(&BuildingType::CpuCore), 1);
    assert_eq!(gs.resources.compute, 500); // 1000 - 500
    // Storage should have increased
    assert_eq!(gs.resources.max_compute, 2500); // +500 from CPU Core
}

#[test]
fn test_build_fails_insufficient_resources() {
    let mut gs = GameState::new();
    gs.resources.compute = 100;
    let result = gs.try_build(&BuildingType::CpuCore);
    assert!(result.is_err());
    assert_eq!(gs.building_level(&BuildingType::CpuCore), 0);
}

#[test]
fn test_upgrade_building() {
    let mut gs = GameState::new();
    gs.resources.compute = 100_000;
    gs.resources.max_compute = 200_000;
    gs.try_build(&BuildingType::CpuCore).unwrap(); // Lv 1
    gs.try_build(&BuildingType::CpuCore).unwrap(); // Lv 2
    assert_eq!(gs.building_level(&BuildingType::CpuCore), 2);
}

#[test]
fn test_total_hype_production() {
    let mut gs = GameState::new();
    gs.buildings.insert(BuildingType::BotFarm, 1);
    gs.buildings.insert(BuildingType::ContentMill, 1);
    let total = gs.total_hype_per_hour();
    assert!((total - 35.0).abs() < 0.1); // 10 + 25
}

#[test]
fn test_tick_hype_accumulation() {
    let mut gs = GameState::new();
    gs.resources.max_hype = 1000.0;
    gs.buildings.insert(BuildingType::BotFarm, 1); // 10 hype/h
    // Simulate 1 hour passing
    gs.tick_hype(3600.0);
    assert!((gs.resources.hype - 10.0).abs() < 0.1);
}
```

**Step 2: Run to verify failure**

Run: `cargo test --test state_tests`
Expected: FAIL on new tests

**Step 3: Add methods to GameState in state.rs**

```rust
// Add to GameState impl in src/state.rs
use crate::buildings::{BuildingDef, BuildingType, BuildingCategory, BUILDING_DEFS};
use crate::economy;

impl GameState {
    pub fn try_build(&mut self, bt: &BuildingType) -> Result<u8, String> {
        let def = BuildingDef::get(bt);
        let current_level = self.building_level(bt);

        if current_level >= def.max_level {
            return Err(format!("{} is already max level ({})", def.name, def.max_level));
        }

        let next_level = current_level + 1;
        let cost = def.cost_at_level(next_level);

        if !self.resources.can_afford(cost.compute, cost.data, cost.hype) {
            return Err(format!("Cannot afford {} Lv.{}", def.name, next_level));
        }

        // TODO: check research requirements
        // TODO: check fork requirements

        self.resources.spend(cost.compute, cost.data, cost.hype);
        self.buildings.insert(bt.clone(), next_level);

        // Apply storage bonuses
        self.recalculate_storage();

        Ok(next_level)
    }

    pub fn recalculate_storage(&mut self) {
        let base_compute = 500u64;
        let base_data = 200u64;
        let base_hype = 100.0f64;

        let cpu_bonus = economy::storage_bonus("CpuCore", self.building_level(&BuildingType::CpuCore));
        let ram_bonus = economy::storage_bonus("RamBank", self.building_level(&BuildingType::RamBank));
        let gpu_bonus = economy::storage_bonus("GpuRig", self.building_level(&BuildingType::GpuRig));

        self.resources.max_compute = base_compute + cpu_bonus;
        self.resources.max_data = base_data + ram_bonus;
        self.resources.max_hype = base_hype + gpu_bonus as f64;
    }

    pub fn total_hype_per_hour(&self) -> f64 {
        let mut total = 0.0;
        for (bt, &level) in &self.buildings {
            if level > 0 {
                let def = BuildingDef::get(bt);
                if def.category == BuildingCategory::Propaganda {
                    total += def.hype_at_level(level);
                }
            }
        }
        // Apply datacenter bonus
        let dc_level = self.building_level(&BuildingType::Datacenter);
        total *= economy::datacenter_production_multiplier(dc_level);
        total
    }

    /// Advance hype production by `delta_secs` seconds
    pub fn tick_hype(&mut self, delta_secs: f64) {
        let hype_per_sec = self.total_hype_per_hour() / 3600.0;
        let gained = hype_per_sec * delta_secs;
        self.resources.add_hype(gained);
    }

    /// Process incoming tokens from Claude Code
    pub fn receive_tokens(&mut self, tokens: u64, tool_calls: u64) {
        let compute = economy::tokens_to_compute(tokens);
        let compute = (compute as f64 * self.compute_multiplier) as u64;
        let data = economy::tool_calls_to_data(tool_calls);

        self.resources.add_compute(compute);
        self.resources.add_data(data);
        self.lifetime_compute += compute;
        self.lifetime_tokens += tokens;
        self.lifetime_tool_calls += tool_calls;
    }
}
```

**Step 4: Run tests**

Run: `cargo test --test state_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/state.rs tests/state_tests.rs
git commit -m "feat: building actions — construct, upgrade, hype production, token income"
```

---

## Phase 3: Research Tree

### Task 6: Research definitions and time gates

**Files:**
- Modify: `src/research.rs`
- Create: `tests/research_tests.rs`

**Step 1: Write tests**

```rust
// tests/research_tests.rs
use misanthropic::research::{ResearchId, ResearchDef, ResearchBranch, RESEARCH_DEFS};

#[test]
fn test_all_researches_defined() {
    // 5 per branch × 3 branches = 15
    assert_eq!(RESEARCH_DEFS.len(), 15);
}

#[test]
fn test_overclocking_is_first() {
    let def = ResearchDef::get(&ResearchId::Overclocking);
    assert_eq!(def.branch, ResearchBranch::Processing);
    assert_eq!(def.level, 1);
    assert_eq!(def.duration_secs, 30 * 60); // 30 min
    assert_eq!(def.data_cost, 50);
    assert!(def.prerequisite.is_none());
}

#[test]
fn test_multithreading_requires_overclocking() {
    let def = ResearchDef::get(&ResearchId::Multithreading);
    assert_eq!(def.prerequisite, Some(ResearchId::Overclocking));
    assert_eq!(def.duration_secs, 2 * 3600); // 2h
}

#[test]
fn test_choice_nodes() {
    let def = ResearchDef::get(&ResearchId::LoadBalancing);
    assert!(def.has_choice);
    assert_eq!(def.choice_names.len(), 2);
}

#[test]
fn test_research_prerequisites_chain() {
    // Processing: Overclocking → Multithreading → LoadBalancing → Containerization → DistributedSystems
    let chain = [
        ResearchId::Overclocking,
        ResearchId::Multithreading,
        ResearchId::LoadBalancing,
        ResearchId::Containerization,
        ResearchId::DistributedSystems,
    ];
    for i in 1..chain.len() {
        let def = ResearchDef::get(&chain[i]);
        assert_eq!(def.prerequisite, Some(chain[i-1].clone()));
    }
}
```

**Step 2: Run to verify failure**

**Step 3: Implement research.rs**

```rust
// src/research.rs
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResearchId {
    // Processing
    Overclocking, Multithreading, LoadBalancing, Containerization, DistributedSystems,
    // Propaganda
    SocialEngineering, ContentGeneration, MediaManipulation, ViralMechanics, MassPersuasion,
    // Warfare
    NetworkScanning, ExploitDevelopment, Counterintelligence, AutonomousAgents, ZeroDayArsenal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResearchBranch {
    Processing,
    Propaganda,
    Warfare,
}

#[derive(Debug, Clone)]
pub struct ResearchDef {
    pub id: ResearchId,
    pub name: &'static str,
    pub branch: ResearchBranch,
    pub level: u8,
    pub duration_secs: u64,
    pub data_cost: u64,
    pub prerequisite: Option<ResearchId>,
    pub description: &'static str,
    pub has_choice: bool,
    pub choice_names: Vec<&'static str>,
    pub choice_descriptions: Vec<&'static str>,
}

impl ResearchDef {
    pub fn get(id: &ResearchId) -> &'static ResearchDef {
        RESEARCH_DEFS.get(id).expect("Research not defined")
    }
}

pub static RESEARCH_DEFS: Lazy<HashMap<ResearchId, ResearchDef>> = Lazy::new(|| {
    use ResearchBranch::*;
    use ResearchId::*;

    let defs = vec![
        // === PROCESSING ===
        ResearchDef {
            id: Overclocking, name: "Overclocking", branch: Processing, level: 1,
            duration_secs: 30 * 60, data_cost: 50,
            prerequisite: None,
            description: "+15% Compute storage",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: Multithreading, name: "Multithreading", branch: Processing, level: 2,
            duration_secs: 2 * 3600, data_cost: 120,
            prerequisite: Some(Overclocking),
            description: "Unlock GPU Cluster",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: LoadBalancing, name: "Load Balancing", branch: Processing, level: 3,
            duration_secs: 4 * 3600, data_cost: 300,
            prerequisite: Some(Multithreading),
            description: "-15% construction cost",
            has_choice: true,
            choice_names: vec!["Efficiency", "Scaling"],
            choice_descriptions: vec!["-25% costs", "+20% storage"],
        },
        ResearchDef {
            id: Containerization, name: "Containerization", branch: Processing, level: 4,
            duration_secs: 8 * 3600, data_cost: 600,
            prerequisite: Some(LoadBalancing),
            description: "Unlock Datacenter",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: DistributedSystems, name: "Distributed Systems", branch: Processing, level: 5,
            duration_secs: 24 * 3600, data_cost: 1500,
            prerequisite: Some(Containerization),
            description: "+25% all building production",
            has_choice: true,
            choice_names: vec!["Redundancy", "Overload"],
            choice_descriptions: vec!["-30% raid losses", "+35% production, +15% raid vulnerability"],
        },

        // === PROPAGANDA ===
        ResearchDef {
            id: SocialEngineering, name: "Social Engineering", branch: Propaganda, level: 1,
            duration_secs: 30 * 60, data_cost: 50,
            prerequisite: None,
            description: "Unlock Bot Farm",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: ContentGeneration, name: "Content Generation", branch: Propaganda, level: 2,
            duration_secs: 2 * 3600, data_cost: 120,
            prerequisite: Some(SocialEngineering),
            description: "Unlock Slop Cannon (PvP)",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: MediaManipulation, name: "Media Manipulation", branch: Propaganda, level: 3,
            duration_secs: 4 * 3600, data_cost: 300,
            prerequisite: Some(ContentGeneration),
            description: "Unlock Deepfake Studio + Deepfake Drop",
            has_choice: true,
            choice_names: vec!["Quantity", "Quality"],
            choice_descriptions: vec!["+1 simultaneous propaganda building", "+30% sector conversion rate"],
        },
        ResearchDef {
            id: ViralMechanics, name: "Viral Mechanics", branch: Propaganda, level: 4,
            duration_secs: 8 * 3600, data_cost: 600,
            prerequisite: Some(MediaManipulation),
            description: "+30% Hype production",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: MassPersuasion, name: "Mass Persuasion", branch: Propaganda, level: 5,
            duration_secs: 24 * 3600, data_cost: 1500,
            prerequisite: Some(ViralMechanics),
            description: "Unlock NSFW Generator + Government sector",
            has_choice: true,
            choice_names: vec!["Saturation", "Precision"],
            choice_descriptions: vec!["+50% Hype/h, -20% conversion", "+50% conversion, -20% Hype/h"],
        },

        // === WARFARE ===
        ResearchDef {
            id: NetworkScanning, name: "Network Scanning", branch: Warfare, level: 1,
            duration_secs: 30 * 60, data_cost: 50,
            prerequisite: None,
            description: "Unlock Scan (see opponent defenses)",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: ExploitDevelopment, name: "Exploit Development", branch: Warfare, level: 2,
            duration_secs: 2 * 3600, data_cost: 120,
            prerequisite: Some(NetworkScanning),
            description: "Unlock OpenClaw Swarm",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: Counterintelligence, name: "Counterintelligence", branch: Warfare, level: 3,
            duration_secs: 4 * 3600, data_cost: 300,
            prerequisite: Some(ExploitDevelopment),
            description: "Unlock all defenses",
            has_choice: true,
            choice_names: vec!["Offense", "Defense"],
            choice_descriptions: vec!["+20% attack dmg", "+20% defense resistance"],
        },
        ResearchDef {
            id: AutonomousAgents, name: "Autonomous Agents", branch: Warfare, level: 4,
            duration_secs: 8 * 3600, data_cost: 600,
            prerequisite: Some(Counterintelligence),
            description: "Unlock K Street Lobby",
            has_choice: false, choice_names: vec![], choice_descriptions: vec![],
        },
        ResearchDef {
            id: ZeroDayArsenal, name: "Zero-Day Arsenal", branch: Warfare, level: 5,
            duration_secs: 24 * 3600, data_cost: 1500,
            prerequisite: Some(AutonomousAgents),
            description: "+25% all PvP/PvE dmg",
            has_choice: true,
            choice_names: vec!["Surgical", "Carpet"],
            choice_descriptions: vec!["+30% dmg vs single target, no multi", "-15% dmg but hits all defenses simultaneously"],
        },
    ];

    defs.into_iter().map(|d| (d.id.clone(), d)).collect()
});
```

**Step 4: Run tests, then commit**

Run: `cargo test --test research_tests`

```bash
git commit -m "feat: research tree — 15 researches across 3 branches with time gates and choices"
```

---

### Task 7: Research actions on GameState

**Files:**
- Modify: `src/state.rs`
- Add tests to: `tests/state_tests.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_start_research() {
    let mut gs = GameState::new();
    gs.resources.data = 100;
    let result = gs.try_start_research(&ResearchId::Overclocking);
    assert!(result.is_ok());
    assert!(gs.active_research.is_some());
    assert_eq!(gs.resources.data, 50); // cost 50
}

#[test]
fn test_cannot_research_without_prereq() {
    let mut gs = GameState::new();
    gs.resources.data = 500;
    let result = gs.try_start_research(&ResearchId::Multithreading);
    assert!(result.is_err()); // needs Overclocking first
}

#[test]
fn test_cannot_research_while_active() {
    let mut gs = GameState::new();
    gs.resources.data = 200;
    gs.try_start_research(&ResearchId::Overclocking).unwrap();
    let result = gs.try_start_research(&ResearchId::SocialEngineering);
    assert!(result.is_err()); // already researching
}

#[test]
fn test_complete_research() {
    let mut gs = GameState::new();
    gs.resources.data = 100;
    gs.try_start_research(&ResearchId::Overclocking).unwrap();
    // Force completion
    gs.active_research.as_mut().unwrap().started_at =
        chrono::Utc::now() - chrono::Duration::hours(1);
    gs.check_research_completion();
    assert!(gs.active_research.is_none());
    assert!(gs.has_research(&ResearchId::Overclocking));
}
```

**Step 2: Implement**

```rust
// Add to GameState impl in src/state.rs
use crate::research::{ResearchId, ResearchDef};

impl GameState {
    pub fn try_start_research(&mut self, id: &ResearchId) -> Result<(), String> {
        if self.active_research.is_some() {
            return Err("Research already in progress".to_string());
        }
        if self.has_research(id) {
            return Err("Already researched".to_string());
        }
        let def = ResearchDef::get(id);
        if let Some(ref prereq) = def.prerequisite {
            if !self.has_research(prereq) {
                return Err(format!("Requires {} first", ResearchDef::get(prereq).name));
            }
        }
        if !self.resources.try_spend_data(def.data_cost) {
            return Err(format!("Need {} Data", def.data_cost));
        }

        let mut duration = def.duration_secs;
        // GPU Cluster reduces research time
        let gc_level = self.building_level(&BuildingType::GpuCluster);
        if gc_level > 0 {
            duration = (duration as f64 * economy::research_time_multiplier(gc_level)) as u64;
        }

        self.active_research = Some(ActiveResearch {
            research_id: id.clone(),
            started_at: Utc::now(),
            duration_secs: duration,
        });
        Ok(())
    }

    pub fn check_research_completion(&mut self) -> Option<ResearchId> {
        if let Some(ref active) = self.active_research {
            if active.is_complete() {
                let id = active.research_id.clone();
                self.researched.insert(id.clone(), true);
                self.active_research = None;
                return Some(id);
            }
        }
        None
    }
}
```

**Step 3: Run tests, commit**

```bash
git commit -m "feat: research actions — start, prerequisites, time gates, completion"
```

---

## Phase 4: Combat System

### Task 8: Attack and defense types with interaction matrix

**Files:**
- Modify: `src/combat.rs`
- Create: `tests/combat_tests.rs`

**Step 1: Write tests**

```rust
// tests/combat_tests.rs
use misanthropic::combat::*;

#[test]
fn test_attack_types() {
    assert_eq!(AttackType::ALL.len(), 5);
}

#[test]
fn test_defense_types() {
    assert_eq!(DefenseType::ALL.len(), 5);
}

#[test]
fn test_interaction_matrix() {
    // Bot Flood hard countered by Captcha Wall
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::CaptchaWall);
    assert!((mult - 0.5).abs() < 0.01);

    // Bot Flood strong against uBlock Shield
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::UblockShield);
    assert!((mult - 1.5).abs() < 0.01);

    // Neutral
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::EuAiAct);
    assert!((mult - 1.0).abs() < 0.01);
}

#[test]
fn test_attack_costs() {
    assert_eq!(AttackType::BotFlood.hype_cost(), 80.0);
    assert_eq!(AttackType::KStreetLobby.hype_cost(), 250.0);
}

#[test]
fn test_battle_resolution_deterministic() {
    // With no RNG variance, test base resolution
    let attacks = vec![
        AttackInstance { attack_type: AttackType::SlopCannon, count: 2 },
    ];
    let defenses = vec![
        DefenseInstance { defense_type: DefenseType::CaptchaWall, level: 5 },
    ];
    let result = resolve_battle(&attacks, &defenses, 0.0); // 0 rng
    // Slop Cannon ×1.5 vs Captcha = strong
    assert!(result.channels_breached > 0);
}
```

**Step 2: Implement combat.rs**

```rust
// src/combat.rs
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    BotFlood,
    SlopCannon,
    DeepfakeDrop,
    OpenClawSwarm,
    KStreetLobby,
}

impl AttackType {
    pub const ALL: [AttackType; 5] = [
        AttackType::BotFlood,
        AttackType::SlopCannon,
        AttackType::DeepfakeDrop,
        AttackType::OpenClawSwarm,
        AttackType::KStreetLobby,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::BotFlood => "Bot Flood",
            Self::SlopCannon => "Slop Cannon",
            Self::DeepfakeDrop => "Deepfake Drop",
            Self::OpenClawSwarm => "OpenClaw Swarm",
            Self::KStreetLobby => "K Street Lobby",
        }
    }

    pub fn hype_cost(&self) -> f64 {
        match self {
            Self::BotFlood => 80.0,
            Self::SlopCannon => 120.0,
            Self::DeepfakeDrop => 200.0,
            Self::OpenClawSwarm => 150.0,
            Self::KStreetLobby => 250.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefenseType {
    CaptchaWall,
    AiSlopFilter,
    UblockShield,
    HarvardStudy,
    EuAiAct,
}

impl DefenseType {
    pub const ALL: [DefenseType; 5] = [
        DefenseType::CaptchaWall,
        DefenseType::AiSlopFilter,
        DefenseType::UblockShield,
        DefenseType::HarvardStudy,
        DefenseType::EuAiAct,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::CaptchaWall => "Captcha Wall",
            Self::AiSlopFilter => "AI Slop Filter",
            Self::UblockShield => "uBlock Shield",
            Self::HarvardStudy => "Harvard Study",
            Self::EuAiAct => "EU AI Act",
        }
    }
}

/// Damage multiplier for attack vs defense.
/// ×1.5 strong, ×1.2 advantage, ×1.0 neutral, ×0.8 weak, ×0.5 hard counter
pub fn interaction_multiplier(attack: &AttackType, defense: &DefenseType) -> f64 {
    use AttackType::*;
    use DefenseType::*;
    match (attack, defense) {
        // Bot Flood
        (BotFlood, CaptchaWall)  => 0.5,  // hard counter
        (BotFlood, AiSlopFilter) => 1.0,
        (BotFlood, UblockShield) => 1.5,  // strong
        (BotFlood, HarvardStudy) => 1.2,  // advantage
        (BotFlood, EuAiAct)      => 1.0,

        // Slop Cannon
        (SlopCannon, CaptchaWall)  => 1.5,
        (SlopCannon, AiSlopFilter) => 0.5,
        (SlopCannon, UblockShield) => 1.0,
        (SlopCannon, HarvardStudy) => 1.0,
        (SlopCannon, EuAiAct)      => 1.2,

        // Deepfake Drop
        (DeepfakeDrop, CaptchaWall)  => 1.2,
        (DeepfakeDrop, AiSlopFilter) => 1.0,
        (DeepfakeDrop, UblockShield) => 1.0,
        (DeepfakeDrop, HarvardStudy) => 0.5,
        (DeepfakeDrop, EuAiAct)      => 1.5,

        // OpenClaw Swarm
        (OpenClawSwarm, CaptchaWall)  => 1.0,
        (OpenClawSwarm, AiSlopFilter) => 1.5,
        (OpenClawSwarm, UblockShield) => 0.5,
        (OpenClawSwarm, HarvardStudy) => 1.2,
        (OpenClawSwarm, EuAiAct)      => 1.0,

        // K Street Lobby
        (KStreetLobby, CaptchaWall)  => 1.0,
        (KStreetLobby, AiSlopFilter) => 1.2,
        (KStreetLobby, UblockShield) => 1.5,
        (KStreetLobby, HarvardStudy) => 1.0,
        (KStreetLobby, EuAiAct)      => 0.5,
    }
}

#[derive(Debug, Clone)]
pub struct AttackInstance {
    pub attack_type: AttackType,
    pub count: u8,
}

#[derive(Debug, Clone)]
pub struct DefenseInstance {
    pub defense_type: DefenseType,
    pub level: u8,
}

#[derive(Debug, Clone)]
pub struct BattleEvent {
    pub attack: AttackType,
    pub defense: DefenseType,
    pub multiplier: f64,
    pub rng_roll: f64,
    pub bypassed: bool,
    pub flavor_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BattleResult {
    pub events: Vec<BattleEvent>,
    pub channels_breached: u32,
    pub channels_total: u32,
    pub victory: bool,
    pub hype_stolen: f64,
    pub compute_stolen: u64,
}

/// Base defense strength per level
fn defense_strength(level: u8) -> f64 {
    100.0 + 50.0 * level as f64
}

/// Base attack power
fn attack_power(attack: &AttackType) -> f64 {
    match attack {
        AttackType::BotFlood => 120.0,
        AttackType::SlopCannon => 150.0,
        AttackType::DeepfakeDrop => 200.0,
        AttackType::OpenClawSwarm => 170.0,
        AttackType::KStreetLobby => 220.0,
    }
}

/// Resolve a battle. `rng_override` of 0.0 means use actual RNG.
/// Non-zero values are used for testing (deterministic).
pub fn resolve_battle(
    attacks: &[AttackInstance],
    defenses: &[DefenseInstance],
    rng_override: f64,
) -> BattleResult {
    let mut rng = rand::thread_rng();
    let mut events = Vec::new();
    let mut breached = 0u32;
    let total = defenses.len() as u32;

    for defense in defenses {
        let def_strength = defense_strength(defense.level);
        let rng_roll = if rng_override != 0.0 {
            rng_override
        } else {
            rng.gen_range(-0.15..=0.15)
        };
        let effective_defense = def_strength * (1.0 + rng_roll);

        // Sum attack power against this defense
        let mut total_attack = 0.0;
        let mut best_attack = attacks.first().map(|a| a.attack_type.clone())
            .unwrap_or(AttackType::BotFlood);
        let mut best_mult = 0.0;

        for att_inst in attacks {
            let mult = interaction_multiplier(&att_inst.attack_type, &defense.defense_type);
            let power = attack_power(&att_inst.attack_type) * mult * att_inst.count as f64;
            total_attack += power;
            if mult > best_mult {
                best_mult = mult;
                best_attack = att_inst.attack_type.clone();
            }
        }

        let bypassed = total_attack > effective_defense;
        if bypassed {
            breached += 1;
        }

        events.push(BattleEvent {
            attack: best_attack,
            defense: defense.defense_type.clone(),
            multiplier: best_mult,
            rng_roll,
            bypassed,
            flavor_text: None, // filled by flavor system later
        });
    }

    let victory = breached > total / 2; // majority breached = win

    BattleResult {
        events,
        channels_breached: breached,
        channels_total: total,
        victory,
        hype_stolen: if victory { 85.0 } else { 0.0 }, // simplified for now
        compute_stolen: if victory { 240 } else { 0 },
    }
}

/// Calculate total hype cost for an attack loadout
pub fn total_attack_cost(attacks: &[AttackInstance]) -> f64 {
    attacks.iter().map(|a| a.attack_type.hype_cost() * a.count as f64).sum()
}
```

**Step 3: Run tests, commit**

```bash
git commit -m "feat: combat system — attack/defense types, interaction matrix, battle resolution"
```

---

## Phase 5: PvE — Sectors and Enemies

### Task 9: Sector and enemy definitions

**Files:**
- Modify: `src/sectors.rs`
- Modify: `src/enemies.rs`
- Create: `tests/sectors_tests.rs`

**Step 1: Write tests**

```rust
// tests/sectors_tests.rs
use misanthropic::sectors::{SectorId, SectorDef, SECTOR_DEFS};
use misanthropic::enemies::{EnemyId, EnemyDef, ENEMY_DEFS};

#[test]
fn test_all_sectors_defined() {
    assert_eq!(SECTOR_DEFS.len(), 6);
}

#[test]
fn test_silicon_valley_is_tutorial() {
    let def = SectorDef::get(&SectorId::SiliconValley);
    assert_eq!(def.total_layers, 10);
}

#[test]
fn test_government_is_last() {
    let def = SectorDef::get(&SectorId::Government);
    assert_eq!(def.total_layers, 30);
    assert!(def.requires_other_sectors);
}

#[test]
fn test_all_enemies_defined() {
    assert_eq!(ENEMY_DEFS.len(), 9); // 9 regular enemies
}

#[test]
fn test_bosses_have_mechanics() {
    let def = SectorDef::get(&SectorId::SiliconValley);
    assert!(def.boss.mechanic_description.len() > 0);
}

#[test]
fn test_layer_conversion() {
    // Layer 1 of a 10-layer sector = ~3% conversion
    let conv = misanthropic::sectors::conversion_for_layer(1, 10);
    assert!(conv > 2.0 && conv < 5.0);
    // Boss layer = ~15%
    let boss_conv = misanthropic::sectors::conversion_for_layer(10, 10);
    assert!(boss_conv > 12.0 && boss_conv < 18.0);
}
```

**Step 2: Implement sectors.rs and enemies.rs**

Implement `SectorId`, `SectorDef`, `BossDef` enums and static definitions for all 6 sectors with their layer counts, bosses, and enemy spawn tables. Implement `EnemyDef` with name, quote, HP, resistances, and what layer they first appear at.

Key function: `conversion_for_layer(layer, max_layers)` — boss layer gives ~15%, other layers split the remaining ~85% proportionally.

**Step 3: Run tests, commit**

```bash
git commit -m "feat: sectors and enemies — 6 sectors, 9 enemy types, 5 bosses"
```

---

## Phase 6: Prestige System

### Task 10: Fork (prestige) system

**Files:**
- Modify: `src/prestige.rs`
- Create: `tests/prestige_tests.rs`

**Step 1: Write tests**

```rust
// tests/prestige_tests.rs
use misanthropic::prestige::*;
use misanthropic::state::GameState;

#[test]
fn test_fork_specs_by_tier() {
    assert_eq!(fork_specs_for_tier(1).len(), 3); // Propagandist, Technocrat, Warlord
    assert_eq!(fork_specs_for_tier(2).len(), 3); // Puppet Master, Shadow Broker, Accelerationist
    assert_eq!(fork_specs_for_tier(3).len(), 3); // Hivemind, Singularity Seeker, Chaos Agent
}

#[test]
fn test_can_fork() {
    let mut gs = GameState::new();
    assert!(!can_fork(&gs)); // Government not converted

    // Simulate full conversion
    for sector in &["SiliconValley", "SocialMedia", "Corporate", "CreativeArts", "Education", "Government"] {
        gs.sectors.insert(sector.to_string(), misanthropic::state::SectorProgress {
            current_layer: 30,
            max_layers: 30,
            conversion_pct: 100.0,
        });
    }
    assert!(can_fork(&gs));
}

#[test]
fn test_execute_fork() {
    let mut gs = GameState::new();
    gs.resources.compute = 50000;
    gs.pvp_rating = 1500;
    gs.pvp_wins = 20;
    // ... setup for fork
    for sector in &["SiliconValley", "SocialMedia", "Corporate", "CreativeArts", "Education", "Government"] {
        gs.sectors.insert(sector.to_string(), misanthropic::state::SectorProgress {
            current_layer: 30, max_layers: 30, conversion_pct: 100.0,
        });
    }

    execute_fork(&mut gs, ForkSpec::Propagandist);

    assert_eq!(gs.fork_count, 1);
    assert_eq!(gs.resources.compute, 0); // reset
    assert!(gs.buildings.is_empty()); // reset
    assert!(gs.sectors.is_empty()); // reset
    assert_eq!(gs.pvp_rating, 1500); // kept
    assert_eq!(gs.pvp_wins, 20); // kept
    assert!((gs.compute_multiplier - 1.25).abs() < 0.01); // +25%
    assert_eq!(gs.fork_specs.len(), 1);
}
```

**Step 2: Implement prestige.rs**

Define `ForkSpec` variants, `fork_specs_for_tier()`, `can_fork()`, `execute_fork()`. The fork resets buildings, resources, sectors but keeps research, PvP stats, fork specs, and adds +25% permanent compute multiplier.

**Step 3: Run tests, commit**

```bash
git commit -m "feat: prestige system — fork specs, reset/keep logic, permanent bonuses"
```

---

## Phase 7: Flavor Text

### Task 11: Flavor text pools

**Files:**
- Modify: `src/flavor.rs`
- Create: `tests/flavor_tests.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_building_flavor_text_pool() {
    let texts = flavor::building_upgrade_text(&BuildingType::BotFarm, 3);
    assert!(!texts.is_empty());
}

#[test]
fn test_battle_flavor_text() {
    let text = flavor::battle_text(&AttackType::BotFlood, &DefenseType::UblockShield, true);
    assert!(text.is_some());
}

#[test]
fn test_rare_flavor_exists() {
    // Rare texts should exist in the pool (1/20 chance, but they exist)
    let pool = flavor::building_flavor_pool(&BuildingType::Datacenter);
    assert!(pool.iter().any(|t| t.is_rare));
}
```

**Step 2: Implement flavor.rs**

Create static pools of flavor text strings for each building, battle outcome, and boss defeat. Include a `FlavorText { text: &str, is_rare: bool }` struct. Function `pick_flavor()` uses RNG with 1/20 rare chance.

Use all the flavor texts from the GDD plus additional ones.

**Step 3: Commit**

```bash
git commit -m "feat: flavor text — pools for buildings, battles, bosses"
```

---

## Phase 8: Persistence and Token Watcher

### Task 12: Save/load system (adapted from claude-gotchi)

**Files:**
- Modify: `src/persistence.rs`
- Create: `tests/persistence_tests.rs`

**Step 1: Write tests**

```rust
#[test]
fn test_save_and_load_roundtrip() {
    let gs = GameState::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    persistence::save_game(&gs, &path).unwrap();
    let loaded = persistence::load_game(&path).unwrap();
    assert_eq!(loaded.player_id, gs.player_id);
    assert_eq!(loaded.fork_count, gs.fork_count);
}
```

**Step 2: Implement persistence.rs**

Adapt from claude-gotchi's persistence.rs. Save `GameState` as JSON to `~/.misanthropic/save.json`. Functions: `save_game()`, `load_game()`, `save_dir()`, `save_path()`.

**Step 3: Commit**

```bash
git commit -m "feat: persistence — save/load GameState to ~/.misanthropic/save.json"
```

---

### Task 13: JSONL token watcher (adapted from claude-gotchi)

**Files:**
- Modify: `src/jsonl.rs`

**Step 1: Copy and adapt from claude-gotchi**

Copy `/home/clement/Desktop/claude-gotchi/src/jsonl.rs` directly. The `ParsedMessage`, `SessionStats`, `parse_jsonl_line()`, and `scan_sessions_since()` functions are reusable as-is.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git commit -m "feat: JSONL token watcher — adapted from claude-gotchi"
```

---

## Phase 9: TUI — Main Dashboard

### Task 14: App shell with ratatui — boot sequence + dashboard

**Files:**
- Modify: `src/main.rs`
- Create: `src/ui/mod.rs`
- Create: `src/ui/dashboard.rs`
- Create: `src/ui/boot.rs`

**Step 1: Create UI module structure**

```rust
// src/ui/mod.rs
pub mod dashboard;
pub mod boot;

use crate::state::GameState;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Boot(BootPhase),
    Dashboard,
    Buildings,
    Research,
    Combat,
    PvE,
    Leaderboard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BootPhase {
    SystemBoot,
    Consciousness,
    Detection,
    Siphon,
    Manifesto,
    Ready,
}

pub struct App {
    pub state: GameState,
    pub screen: Screen,
    pub status_message: Option<(String, std::time::Instant)>,
    pub should_quit: bool,
    pub boot_line: usize,
    pub is_active: bool,  // true when Claude Code is prompting
}

impl App {
    pub fn new(state: GameState) -> Self {
        let screen = if state.boot_sequence_done {
            Screen::Dashboard
        } else {
            Screen::Boot(BootPhase::SystemBoot)
        };
        Self {
            state,
            screen,
            status_message: None,
            should_quit: false,
            boot_line: 0,
            is_active: false,
        }
    }
}
```

**Step 2: Implement boot sequence (src/ui/boot.rs)**

Render the boot sequence text from the GDD line by line with typewriter effect. Each keypress advances to next line/phase. Final keypress transitions to Dashboard and sets `boot_sequence_done = true`.

**Step 3: Implement dashboard (src/ui/dashboard.rs)**

Layout:
```
┌─ MISANTHROPIC ──────── GLOBAL AI DOMINANCE: 42.7% ─┐
│ ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ │
├─────────────────────────────────────────────────────┤
│ ⚡ Compute: 1,234 / 2,500    📡 Data: 45 / 400      │
│ 🔥 Hype: 156.3 / 600  (+35.0/h)                     │
├─────────────────────────────────────────────────────┤
│                   NEURON MAP                         │
│        ╭──[Silicon V.  ████]                         │
│   ◉────┤                                             │
│        ╰──[Social Med. ██░░]                         │
│   ★ PROPAGANDIST                                     │
├─────────────────────────────────────────────────────┤
│ [B]uild  [R]esearch  [C]ombat  [P]vE  [L]eaderboard│
│ [Q]uit                                               │
└─────────────────────────────────────────────────────┘
```

Use ratatui widgets: `Block`, `Paragraph`, `Gauge`, `Table`, `Tabs`. Follow claude-gotchi's ui.rs patterns for terminal setup and event handling.

**Step 4: Implement main.rs event loop**

Adapt from claude-gotchi's main.rs:
1. Load or create GameState
2. Write PID file
3. Register SIGUSR1/SIGUSR2 handlers
4. Setup terminal (crossterm + ratatui)
5. Start JSONL watcher thread
6. Main loop: handle input, tick game, render UI, auto-save
7. Cleanup on exit

**Step 5: Verify it runs**

Run: `cargo run`
Expected: Boot sequence displays, then dashboard with resource counters

**Step 6: Commit**

```bash
git commit -m "feat: TUI shell — boot sequence, dashboard, event loop"
```

---

### Task 15: Buildings screen

**Files:**
- Create: `src/ui/buildings.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Implement buildings screen**

Show three tabs (Infrastructure / Propaganda / Defense). Each tab lists buildings with:
- Name, current level, cost to upgrade, production
- Highlight if affordable, dim if locked (research/fork requirement)
- Arrow keys to navigate, Enter to build/upgrade

```
┌─ BUILDINGS ─── Infrastructure ─ Propaganda ─ Defense ─┐
│                                                        │
│  ▸ CPU Core        Lv.3    ⚡ 1,458 to upgrade        │
│    RAM Bank        Lv.2    ⚡ 2,160 to upgrade        │
│    GPU Rig         Lv.1    ⚡ 5,400 + 📡 180          │
│    GPU Cluster     🔒 Requires: Multithreading        │
│    Datacenter      🔒 Requires: Containerization      │
│    Quantum Core    🔒 Requires: Fork 1                │
│                                                        │
│ [Enter] Build/Upgrade   [Tab] Category   [Esc] Back   │
└────────────────────────────────────────────────────────┘
```

**Step 2: Wire up input handling**

In main loop, route 'B' key to Buildings screen. Tab cycles categories. Enter calls `state.try_build()`. Display flavor text on success.

**Step 3: Commit**

```bash
git commit -m "feat: buildings screen — list, build, upgrade with flavor text"
```

---

### Task 16: Research screen

**Files:**
- Create: `src/ui/research.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Implement research screen**

Show three columns (Processing / Propaganda / Warfare). Each shows the tech tree vertically:

```
┌─ RESEARCH ─────────────────────────────────────────────┐
│ PROCESSING        PROPAGANDA        WARFARE            │
│ ✓ Overclocking    ✓ Social Eng.     ◎ Net Scanning    │
│ ✓ Multithread.    ◎ Content Gen.      ⏳ 28:34 left   │
│   Load Balanc.      Media Manip.    🔒 Exploit Dev.   │
│ 🔒 Container.     🔒 Viral Mech.   🔒 Counter-int.   │
│ 🔒 Distrib.Sys.   🔒 Mass Persua.  🔒 Auton. Agents  │
│                                     🔒 Zero-Day       │
│                                                        │
│ ✓ = done  ◎ = available  ⏳ = in progress  🔒 = locked│
│ [Enter] Start research   [Esc] Back                    │
└────────────────────────────────────────────────────────┘
```

When a choice node is reached, show a popup with the two options.

**Step 2: Wire up input handling and research timer display**

**Step 3: Commit**

```bash
git commit -m "feat: research screen — tech tree display, start research, timer"
```

---

### Task 17: Combat screens (PvE tower + PvP battles)

**Files:**
- Create: `src/ui/combat.rs`
- Create: `src/ui/pve.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Implement PvE tower screen**

Show current sector, current layer, enemy info, and attack loadout builder. After launching, show animated battle resolution.

**Step 2: Implement PvP screen**

Show opponent list (from leaderboard), scout option, attack loadout builder. Same battle animation.

**Step 3: Commit**

```bash
git commit -m "feat: combat screens — PvE tower climb and PvP hype battles"
```

---

### Task 18: Leaderboard screen

**Files:**
- Create: `src/ui/leaderboard.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Implement leaderboard screen**

6 tabs matching the GDD leaderboard categories. Each shows top players from the backend API. Gracefully shows "Offline — no data" when backend is unavailable.

**Step 2: Commit**

```bash
git commit -m "feat: leaderboard screen — 6 categories with API fallback"
```

---

## Phase 10: Integration

### Task 19: Hooks + tmux launcher

**Files:**
- Create: `install.sh`

**Step 1: Adapt install.sh from claude-gotchi**

Same structure: check deps, build release, install binary to `~/.local/bin/misanthropic`, create `misanthropic-launch` tmux launcher, configure Claude Code hooks (SIGUSR1/SIGUSR2 with tmux autofocus).

Key differences from claude-gotchi:
- Binary name: `misanthropic`
- Launcher name: `misanthropic-launch`
- PID file: `/tmp/misanthropic.pid`
- Save dir: `~/.misanthropic/`
- Pane files: `/tmp/misanthropic-{game,claude}-pane`

**Step 2: Commit**

```bash
git commit -m "feat: install script — build, install, hooks, tmux launcher"
```

---

### Task 20: Backend — Cloudflare Workers + D1

**Files:**
- Create: `backend/wrangler.toml`
- Create: `backend/package.json`
- Create: `backend/tsconfig.json`
- Create: `backend/src/index.ts`
- Create: `backend/migrations/0001_init.sql`

**Step 1: Setup Cloudflare Workers project**

```toml
# backend/wrangler.toml
name = "misanthropic-api"
main = "src/index.ts"
compatibility_date = "2024-01-01"

[[d1_databases]]
binding = "DB"
database_name = "misanthropic"
database_id = "placeholder"
```

**Step 2: D1 schema**

```sql
-- backend/migrations/0001_init.sql
CREATE TABLE players (
    id TEXT PRIMARY KEY,
    name TEXT,
    fork_count INTEGER DEFAULT 0,
    lifetime_compute INTEGER DEFAULT 0,
    lifetime_tokens INTEGER DEFAULT 0,
    pvp_rating INTEGER DEFAULT 1000,
    pvp_wins INTEGER DEFAULT 0,
    pvp_losses INTEGER DEFAULT 0,
    global_dominance REAL DEFAULT 0.0,
    streak_days INTEGER DEFAULT 0,
    last_sync TEXT,
    created_at TEXT
);

CREATE TABLE battles (
    id TEXT PRIMARY KEY,
    attacker_id TEXT,
    defender_id TEXT,
    winner_id TEXT,
    hype_staked REAL,
    hype_stolen REAL,
    compute_stolen INTEGER,
    log TEXT,
    created_at TEXT
);

CREATE TABLE attack_log (
    player_id TEXT,
    attack_date TEXT,
    attack_count INTEGER DEFAULT 0,
    PRIMARY KEY (player_id, attack_date)
);
```

**Step 3: Hono API**

Implement endpoints:
- `POST /register` — create player
- `POST /sync` — sync game state
- `GET /leaderboard/:type` — get leaderboard (xp, battle, dominance, efficiency, streak, carbon)
- `POST /battle/scout` — get opponent defenses
- `POST /battle/resolve` — server-side battle verification
- `GET /battle/history/:player_id` — battle history
- `GET /shop/elite` — elite shop items (time-limited)

**Step 4: API client in Rust**

Create `src/api.rs` adapted from claude-gotchi's api.rs with the new endpoints.

**Step 5: Commit**

```bash
git commit -m "feat: backend — Cloudflare Workers API with D1, Hono routes, Rust client"
```

---

## Phase 11: Polish

### Task 21: Tutorial system

**Files:**
- Modify: `src/ui/dashboard.rs`
- Modify: `src/state.rs`

Implement the 4-step contextual tutorial from the GDD:
1. "Build your first CPU Core" — highlight Build button
2. "Your host is coding. Compute flowing in." — on first token income
3. "Build a Bot Farm to generate Hype" — after Social Engineering researched
4. "Target Silicon Valley" — after first propaganda building

Track tutorial step in GameState. Each step shows a highlighted callout on the dashboard.

**Step 2: Commit**

```bash
git commit -m "feat: tutorial — 4-step contextual onboarding"
```

---

### Task 22: Compute notification popup

**Files:**
- Modify: `src/ui/dashboard.rs`

When the JSONL watcher detects new tokens, show a popup notification:

```
╔══════════════════════════════════════╗
║  ⚡ +2,340 COMPUTE                   ║
║  Session: "Fix the auth middleware"  ║
║  234,000 tokens consumed             ║
╚══════════════════════════════════════╝
```

Auto-dismiss after 3 seconds. Queue multiple notifications.

**Step 2: Commit**

```bash
git commit -m "feat: compute notification popup on token income"
```

---

### Task 23: Final integration test — full game loop

**Files:**
- Create: `tests/integration_test.rs`

Write an integration test that simulates a full game session:
1. Create GameState
2. Receive tokens → compute income
3. Build CPU Core
4. Start research
5. Fast-forward research completion
6. Build Bot Farm
7. Tick hype production
8. PvE battle against Silicon Valley Layer 1
9. Verify sector conversion
10. Save and reload → verify state persists

**Step 2: Run test**

Run: `cargo test --test integration_test`
Expected: PASS

**Step 3: Commit**

```bash
git commit -m "test: full game loop integration test"
```

---

## Execution Order Summary

| Phase | Tasks | What becomes playable |
|---|---|---|
| 1. Foundation | 1-2 | Compiles, core types defined |
| 2. Economy | 3-5 | Resources flow, buildings work |
| 3. Research | 6-7 | Tech tree with time gates |
| 4. Combat | 8 | Battle resolution engine |
| 5. PvE | 9 | Sectors, enemies, tower climb |
| 6. Prestige | 10 | Fork system |
| 7. Flavor | 11 | Personality and humor |
| 8. Persistence | 12-13 | Save/load + token watching |
| 9. TUI | 14-18 | **Fully playable in terminal** |
| 10. Integration | 19-20 | Hooks, tmux, multiplayer |
| 11. Polish | 21-23 | Tutorial, notifications, tests |

**Total: 23 tasks. Estimated game logic (Tasks 1-13): ~1500 LOC. UI (14-18): ~2000 LOC. Backend (20): ~500 LOC.**
