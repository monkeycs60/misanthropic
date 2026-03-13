use misanthropic::buildings::{BuildingCategory, BuildingDef, BuildingType, BUILDING_DEFS};

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
