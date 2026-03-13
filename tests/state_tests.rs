use misanthropic::state::{Resources, GameState};
use misanthropic::buildings::BuildingType;

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

#[test]
fn test_build_cpu_core() {
    let mut gs = GameState::new();
    gs.resources.compute = 1000;
    gs.resources.max_compute = 2000;
    let result = gs.try_build(&BuildingType::CpuCore);
    assert!(result.is_ok());
    assert_eq!(gs.building_level(&BuildingType::CpuCore), 1);
    assert_eq!(gs.resources.compute, 500); // 1000 - 500
    assert_eq!(gs.resources.max_compute, 1000); // base 500 + 500 from CPU Core Lv1
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
    gs.try_build(&BuildingType::CpuCore).unwrap();
    gs.try_build(&BuildingType::CpuCore).unwrap();
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
    gs.buildings.insert(BuildingType::BotFarm, 1);
    gs.tick_hype(3600.0);
    assert!((gs.resources.hype - 10.0).abs() < 0.1);
}

#[test]
fn test_receive_tokens() {
    let mut gs = GameState::new();
    gs.resources.max_compute = 10000;
    gs.resources.max_data = 1000;
    gs.receive_tokens(234_000, 50);
    assert_eq!(gs.resources.compute, 2340);
    assert_eq!(gs.resources.data, 50);
    assert_eq!(gs.lifetime_tokens, 234_000);
    assert_eq!(gs.lifetime_tool_calls, 50);
}
