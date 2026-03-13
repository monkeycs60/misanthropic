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
    let base = 500u64;
    let lv1 = economy::building_cost(base, 1);
    let lv2 = economy::building_cost(base, 2);
    let lv5 = economy::building_cost(base, 5);
    assert_eq!(lv1, 500);
    assert!(lv2 > lv1);
    assert!(lv5 > lv2);
    assert_eq!(lv2, 900); // 500 * 1.8
}

#[test]
fn test_hype_production_scaling() {
    let base = 10.0f64;
    let lv1 = economy::hype_per_hour(base, 1);
    let lv3 = economy::hype_per_hour(base, 3);
    assert!((lv1 - 10.0).abs() < 0.01);
    assert!((lv3 - 19.6).abs() < 0.1); // 10 * 1.4^2
}

#[test]
fn test_compute_multiplier_from_forks() {
    assert!((economy::fork_compute_multiplier(0) - 1.0).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(1) - 1.25).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(2) - 1.5).abs() < 0.01);
    assert!((economy::fork_compute_multiplier(3) - 1.75).abs() < 0.01);
}
