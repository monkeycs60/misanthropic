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
