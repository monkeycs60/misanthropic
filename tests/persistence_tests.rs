use misanthropic::persistence;
use misanthropic::state::GameState;

#[test]
fn test_save_and_load_roundtrip() {
    let gs = GameState::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    persistence::save_game(&gs, &path).unwrap();
    let loaded = persistence::load_game(&path).unwrap();
    assert_eq!(loaded.player_id, gs.player_id);
    assert_eq!(loaded.fork_count, gs.fork_count);
    assert_eq!(loaded.pvp_rating, gs.pvp_rating);
}

#[test]
fn test_save_creates_directories() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/dir/save.json");
    let gs = GameState::new();
    persistence::save_game(&gs, &path).unwrap();
    assert!(path.exists());
}

#[test]
fn test_load_nonexistent_fails() {
    let result = persistence::load_game(std::path::Path::new("/tmp/does_not_exist_12345.json"));
    assert!(result.is_err());
}

#[test]
fn test_save_path_contains_misanthropic() {
    let path = persistence::save_path();
    assert!(path.to_string_lossy().contains(".misanthropic"));
    assert!(path.to_string_lossy().ends_with("save.json"));
}

#[test]
fn test_roundtrip_preserves_resources() {
    let mut gs = GameState::new();
    gs.resources.compute = 42;
    gs.resources.data = 99;
    gs.resources.hype = 50.5;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    persistence::save_game(&gs, &path).unwrap();
    let loaded = persistence::load_game(&path).unwrap();

    assert_eq!(loaded.resources.compute, 42);
    assert_eq!(loaded.resources.data, 99);
    assert!((loaded.resources.hype - 50.5).abs() < 0.01);
}
