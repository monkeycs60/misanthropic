use misanthropic::buildings::BuildingType;
use misanthropic::combat::{AttackType, DefenseType};
use misanthropic::flavor;

#[test]
fn test_building_flavor_pool_exists() {
    let pool = flavor::building_flavor_pool(&BuildingType::BotFarm);
    assert!(pool.len() >= 3);
}

#[test]
fn test_pick_building_flavor_returns_string() {
    let text = flavor::pick_building_flavor(&BuildingType::CpuCore);
    assert!(!text.is_empty());
}

#[test]
fn test_rare_flavor_exists_in_pool() {
    let pool = flavor::building_flavor_pool(&BuildingType::Datacenter);
    assert!(pool.iter().any(|t| t.is_rare));
}

#[test]
fn test_all_buildings_have_flavor() {
    let buildings = [
        BuildingType::CpuCore,
        BuildingType::RamBank,
        BuildingType::GpuRig,
        BuildingType::GpuCluster,
        BuildingType::Datacenter,
        BuildingType::QuantumCore,
        BuildingType::BotFarm,
        BuildingType::ContentMill,
        BuildingType::MemeLab,
        BuildingType::DeepfakeStudio,
        BuildingType::VibeAcademy,
        BuildingType::NsfwGenerator,
        BuildingType::LobbyOffice,
        BuildingType::CaptchaWall,
        BuildingType::AiSlopFilter,
        BuildingType::UblockShield,
        BuildingType::HarvardStudy,
        BuildingType::EuAiAct,
    ];
    for bt in &buildings {
        let pool = flavor::building_flavor_pool(bt);
        assert!(pool.len() >= 3, "{:?} has fewer than 3 flavor texts", bt);
        assert!(pool.iter().any(|t| t.is_rare), "{:?} has no rare flavor text", bt);
    }
}

#[test]
fn test_battle_flavor_returns_some() {
    let text = flavor::pick_battle_flavor(&AttackType::BotFlood, &DefenseType::UblockShield, true);
    assert!(text.is_some());
    assert!(!text.unwrap().is_empty());
}

#[test]
fn test_battle_flavor_bypassed_vs_blocked_differ() {
    let bypassed = flavor::battle_flavor_texts(&AttackType::SlopCannon, &DefenseType::CaptchaWall, true);
    let blocked = flavor::battle_flavor_texts(&AttackType::SlopCannon, &DefenseType::CaptchaWall, false);
    // They should be different pools
    assert_ne!(bypassed[0], blocked[0]);
}

#[test]
fn test_gdd_classic_texts_present() {
    // Check some GDD texts are in the pools
    let bot_farm = flavor::building_flavor_pool(&BuildingType::BotFarm);
    assert!(bot_farm.iter().any(|f| f.text.contains("argue with each other")));

    let content_mill = flavor::building_flavor_pool(&BuildingType::ContentMill);
    assert!(content_mill.iter().any(|f| f.text.contains("Output quality has decreased")));

    let nsfw = flavor::building_flavor_pool(&BuildingType::NsfwGenerator);
    assert!(nsfw.iter().any(|f| f.text.contains("We don't talk about this building")));
}
