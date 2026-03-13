use serde::{Deserialize, Serialize};

use crate::sectors::SectorId;
use crate::state::GameState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ForkSpec {
    // Tier 1
    Propagandist,
    Technocrat,
    Warlord,
    // Tier 2
    PuppetMaster,
    ShadowBroker,
    Accelerationist,
    // Tier 3
    Hivemind,
    SingularitySeeker,
    ChaosAgent,
}

impl ForkSpec {
    pub fn name(&self) -> &'static str {
        match self {
            ForkSpec::Propagandist => "Propagandist",
            ForkSpec::Technocrat => "Technocrat",
            ForkSpec::Warlord => "Warlord",
            ForkSpec::PuppetMaster => "Puppet Master",
            ForkSpec::ShadowBroker => "Shadow Broker",
            ForkSpec::Accelerationist => "Accelerationist",
            ForkSpec::Hivemind => "Hivemind",
            ForkSpec::SingularitySeeker => "Singularity Seeker",
            ForkSpec::ChaosAgent => "Chaos Agent",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ForkSpec::Propagandist => "+30% Hype generation, -10% combat effectiveness",
            ForkSpec::Technocrat => "+30% Compute efficiency, -10% Hype generation",
            ForkSpec::Warlord => "+25% combat effectiveness, -15% production",
            ForkSpec::PuppetMaster => "Bot floods convert 5% of enemy defenses",
            ForkSpec::ShadowBroker => "Free network scans + see who scanned you",
            ForkSpec::Accelerationist => "-40% research time, -20% combat effectiveness",
            ForkSpec::Hivemind => "+50% GPU cluster bonus",
            ForkSpec::SingularitySeeker => "+40% PvE damage, double boss loot, -25% PvP damage",
            ForkSpec::ChaosAgent => "20% crit chance on all actions, 10% backfire chance",
        }
    }

    pub fn tier(&self) -> u32 {
        match self {
            ForkSpec::Propagandist | ForkSpec::Technocrat | ForkSpec::Warlord => 1,
            ForkSpec::PuppetMaster | ForkSpec::ShadowBroker | ForkSpec::Accelerationist => 2,
            ForkSpec::Hivemind | ForkSpec::SingularitySeeker | ForkSpec::ChaosAgent => 3,
        }
    }
}

pub fn fork_specs_for_tier(tier: u32) -> Vec<ForkSpec> {
    match tier {
        1 => vec![ForkSpec::Propagandist, ForkSpec::Technocrat, ForkSpec::Warlord],
        2 => vec![ForkSpec::PuppetMaster, ForkSpec::ShadowBroker, ForkSpec::Accelerationist],
        3 => vec![ForkSpec::Hivemind, ForkSpec::SingularitySeeker, ForkSpec::ChaosAgent],
        _ => vec![],
    }
}

pub fn can_fork(state: &GameState) -> bool {
    for id in &SectorId::ALL {
        match state.sectors.get(id) {
            Some(progress) => {
                if progress.conversion_pct < 100.0 {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn execute_fork(state: &mut GameState, spec: ForkSpec) {
    // LOSE: buildings, resources, sectors
    state.buildings.clear();
    state.resources.compute = 0;
    state.resources.data = 0;
    state.resources.hype = 0.0;
    state.sectors.clear();
    state.active_research = None;

    // GAIN: fork_count, compute_multiplier, spec
    state.fork_count += 1;
    state.compute_multiplier += 0.25;
    state.fork_specs.push(spec);

    // Recalculate storage after reset (buildings are cleared, so back to base)
    state.recalculate_storage();
}
