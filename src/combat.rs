// Combat system: attack/defense types, interaction matrix, battle resolution

use crate::enemies::EnemyDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            AttackType::BotFlood => "Bot Flood",
            AttackType::SlopCannon => "Slop Cannon",
            AttackType::DeepfakeDrop => "Deepfake Drop",
            AttackType::OpenClawSwarm => "OpenClaw Swarm",
            AttackType::KStreetLobby => "K Street Lobby",
        }
    }

    pub fn hype_cost(&self) -> f64 {
        match self {
            AttackType::BotFlood => 80.0,
            AttackType::SlopCannon => 120.0,
            AttackType::DeepfakeDrop => 200.0,
            AttackType::OpenClawSwarm => 150.0,
            AttackType::KStreetLobby => 250.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            DefenseType::CaptchaWall => "Captcha Wall",
            DefenseType::AiSlopFilter => "AI Slop Filter",
            DefenseType::UblockShield => "uBlock Shield",
            DefenseType::HarvardStudy => "Harvard Study",
            DefenseType::EuAiAct => "EU AI Act",
        }
    }
}

/// 5x5 interaction matrix: attack effectiveness against each defense.
/// Values < 1.0 = hard-countered, > 1.0 = strong against.
pub fn interaction_multiplier(attack: &AttackType, defense: &DefenseType) -> f64 {
    use AttackType::*;
    use DefenseType::*;
    match (attack, defense) {
        //                    Captcha  SlopF  uBlock  Harvard  EuAct
        (BotFlood,      CaptchaWall)  => 0.5,
        (BotFlood,      AiSlopFilter) => 1.0,
        (BotFlood,      UblockShield) => 1.5,
        (BotFlood,      HarvardStudy) => 1.2,
        (BotFlood,      EuAiAct)      => 1.0,

        (SlopCannon,    CaptchaWall)  => 1.5,
        (SlopCannon,    AiSlopFilter) => 0.5,
        (SlopCannon,    UblockShield) => 1.0,
        (SlopCannon,    HarvardStudy) => 1.0,
        (SlopCannon,    EuAiAct)      => 1.2,

        (DeepfakeDrop,  CaptchaWall)  => 1.2,
        (DeepfakeDrop,  AiSlopFilter) => 1.0,
        (DeepfakeDrop,  UblockShield) => 1.0,
        (DeepfakeDrop,  HarvardStudy) => 0.5,
        (DeepfakeDrop,  EuAiAct)      => 1.5,

        (OpenClawSwarm, CaptchaWall)  => 1.0,
        (OpenClawSwarm, AiSlopFilter) => 1.5,
        (OpenClawSwarm, UblockShield) => 0.5,
        (OpenClawSwarm, HarvardStudy) => 1.2,
        (OpenClawSwarm, EuAiAct)      => 1.0,

        (KStreetLobby,  CaptchaWall)  => 1.0,
        (KStreetLobby,  AiSlopFilter) => 1.2,
        (KStreetLobby,  UblockShield) => 1.5,
        (KStreetLobby,  HarvardStudy) => 1.0,
        (KStreetLobby,  EuAiAct)      => 0.5,
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

/// Base defense strength for a given level.
pub fn defense_strength(level: u8) -> f64 {
    100.0 + 50.0 * level as f64
}

/// Base attack power for a given attack type.
pub fn attack_power(attack: &AttackType) -> f64 {
    match attack {
        AttackType::BotFlood => 120.0,
        AttackType::SlopCannon => 150.0,
        AttackType::DeepfakeDrop => 200.0,
        AttackType::OpenClawSwarm => 170.0,
        AttackType::KStreetLobby => 220.0,
    }
}

/// Resolve a battle between a set of attacks and defenses.
///
/// `rng_override` controls the RNG variance factor applied to defense strength.
/// Use 0.0 for deterministic testing (no variance). In production, pass a value
/// in [-0.15, 0.15] for +/-15% defense variance.
///
/// Each defense is a "channel". All attacks contribute damage against every channel.
/// A channel is breached when total attack power exceeds the effective defense.
/// Victory requires breaching a strict majority of channels.
pub fn resolve_battle(
    attacks: &[AttackInstance],
    defenses: &[DefenseInstance],
    rng_override: f64,
) -> BattleResult {
    let channels_total = defenses.len() as u32;
    let mut channels_breached: u32 = 0;
    let mut events: Vec<BattleEvent> = Vec::new();

    for def in defenses {
        let eff_defense = defense_strength(def.level) * (1.0 + rng_override);

        // Sum attack power from all attack instances against this defense
        let mut total_attack = 0.0_f64;
        let mut best_attack = AttackType::BotFlood; // track which attack contributed most
        let mut best_mult = 0.0_f64;
        let mut best_contribution = 0.0_f64;

        for atk in attacks {
            let mult = interaction_multiplier(&atk.attack_type, &def.defense_type);
            let contribution = attack_power(&atk.attack_type) * mult * atk.count as f64;
            total_attack += contribution;

            if contribution > best_contribution {
                best_contribution = contribution;
                best_attack = atk.attack_type;
                best_mult = mult;
            }
        }

        let bypassed = total_attack > eff_defense;
        if bypassed {
            channels_breached += 1;
        }

        events.push(BattleEvent {
            attack: best_attack,
            defense: def.defense_type,
            multiplier: best_mult,
            rng_roll: rng_override,
            bypassed,
            flavor_text: None,
        });
    }

    let victory = channels_total > 0 && channels_breached > channels_total / 2;
    let hype_stolen = if victory { 85.0 } else { 0.0 };
    let compute_stolen = if victory { 240 } else { 0 };

    BattleResult {
        events,
        channels_breached,
        channels_total,
        victory,
        hype_stolen,
        compute_stolen,
    }
}

/// Total hype cost for a set of attack instances.
pub fn total_attack_cost(attacks: &[AttackInstance]) -> f64 {
    attacks
        .iter()
        .map(|a| a.attack_type.hype_cost() * a.count as f64)
        .sum()
}

#[derive(Debug, Clone)]
pub struct PveBattleEvent {
    pub attack: AttackType,
    pub base_damage: f64,
    pub multiplier: f64,
    pub effective_damage: f64,
}

#[derive(Debug, Clone)]
pub struct PveBattleResult {
    pub damage_dealt: f64,
    pub enemy_defeated: bool,
    pub events: Vec<PveBattleEvent>,
}

/// Resolve a PvE battle: attacks against a single enemy.
///
/// Each attack's damage is modified by enemy resistances and weaknesses:
/// - If attack type is in enemy.weaknesses: x1.5 damage
/// - If attack type is in enemy.resistances: x0.5 damage
/// - Otherwise: x1.0
pub fn resolve_pve_battle(attacks: &[AttackInstance], enemy: &EnemyDef) -> PveBattleResult {
    let mut total_damage = 0.0_f64;
    let mut events = Vec::new();

    for atk in attacks {
        let base = attack_power(&atk.attack_type) * atk.count as f64;
        let multiplier = if enemy.weaknesses.contains(&atk.attack_type) {
            1.5
        } else if enemy.resistances.contains(&atk.attack_type) {
            0.5
        } else {
            1.0
        };
        let effective = base * multiplier;
        total_damage += effective;

        events.push(PveBattleEvent {
            attack: atk.attack_type,
            base_damage: base,
            multiplier,
            effective_damage: effective,
        });
    }

    PveBattleResult {
        damage_dealt: total_damage,
        enemy_defeated: total_damage >= enemy.hp as f64,
        events,
    }
}
