use misanthropic::prestige::*;
use misanthropic::sectors::SectorId;
use misanthropic::state::{GameState, SectorProgress};

#[test]
fn test_fork_specs_by_tier() {
    assert_eq!(fork_specs_for_tier(1).len(), 3);
    assert_eq!(fork_specs_for_tier(2).len(), 3);
    assert_eq!(fork_specs_for_tier(3).len(), 3);
    assert_eq!(fork_specs_for_tier(4).len(), 0); // no tier 4
}

#[test]
fn test_fork_spec_names() {
    assert_eq!(ForkSpec::Propagandist.name(), "Propagandist");
    assert_eq!(ForkSpec::ChaosAgent.name(), "Chaos Agent");
}

#[test]
fn test_fork_spec_tiers() {
    assert_eq!(ForkSpec::Propagandist.tier(), 1);
    assert_eq!(ForkSpec::PuppetMaster.tier(), 2);
    assert_eq!(ForkSpec::Hivemind.tier(), 3);
}

#[test]
fn test_can_fork_requires_all_sectors() {
    let mut gs = GameState::new();
    assert!(!can_fork(&gs));

    // Add all 6 sectors at 100%
    for id in &SectorId::ALL {
        gs.sectors.insert(id.clone(), SectorProgress {
            current_layer: 30, max_layers: 30, conversion_pct: 100.0,
        });
    }
    assert!(can_fork(&gs));
}

#[test]
fn test_can_fork_fails_without_government() {
    let mut gs = GameState::new();
    for id in &[SectorId::SiliconValley, SectorId::SocialMedia, SectorId::Corporate, SectorId::CreativeArts, SectorId::Education] {
        gs.sectors.insert(id.clone(), SectorProgress {
            current_layer: 30, max_layers: 30, conversion_pct: 100.0,
        });
    }
    assert!(!can_fork(&gs)); // missing Government
}

#[test]
fn test_execute_fork() {
    let mut gs = GameState::new();
    gs.resources.compute = 50000;
    gs.pvp_rating = 1500;
    gs.pvp_wins = 20;
    gs.buildings.insert(misanthropic::buildings::BuildingType::CpuCore, 5);
    gs.researched.insert(misanthropic::research::ResearchId::Overclocking, true);

    for id in &SectorId::ALL {
        gs.sectors.insert(id.clone(), SectorProgress {
            current_layer: 30, max_layers: 30, conversion_pct: 100.0,
        });
    }

    execute_fork(&mut gs, ForkSpec::Propagandist);

    // LOSE
    assert_eq!(gs.resources.compute, 0);
    assert!(gs.buildings.is_empty());
    assert!(gs.sectors.is_empty());

    // KEEP
    assert_eq!(gs.pvp_rating, 1500);
    assert_eq!(gs.pvp_wins, 20);
    assert!(gs.has_research(&misanthropic::research::ResearchId::Overclocking));

    // GAIN
    assert_eq!(gs.fork_count, 1);
    assert!((gs.compute_multiplier - 1.25).abs() < 0.01);
    assert_eq!(gs.fork_specs.len(), 1);
}
