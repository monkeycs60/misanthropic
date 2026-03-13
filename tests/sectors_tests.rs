use misanthropic::sectors::*;
use misanthropic::enemies::*;

#[test]
fn test_all_sectors_defined() {
    assert_eq!(SECTOR_DEFS.len(), 6);
}

#[test]
fn test_silicon_valley_is_tutorial() {
    let def = SectorDef::get(&SectorId::SiliconValley);
    assert_eq!(def.total_layers, 10);
    assert!(!def.requires_other_sectors);
}

#[test]
fn test_government_is_last() {
    let def = SectorDef::get(&SectorId::Government);
    assert_eq!(def.total_layers, 30);
    assert!(def.requires_other_sectors);
}

#[test]
fn test_all_enemies_defined() {
    assert_eq!(ENEMY_DEFS.len(), 9);
}

#[test]
fn test_bosses_have_mechanics() {
    let def = SectorDef::get(&SectorId::SiliconValley);
    assert!(!def.boss.mechanic_description.is_empty());
}

#[test]
fn test_layer_conversion() {
    let conv = conversion_for_layer(1, 10);
    assert!(conv > 2.0 && conv < 15.0);
    let boss_conv = conversion_for_layer(10, 10);
    assert!((boss_conv - 15.0).abs() < 0.01);
}

#[test]
fn test_enemies_for_layer() {
    let early = enemies_for_layer(1);
    assert!(early.iter().any(|e| e.id == EnemyId::JuniorSkeptic));
    let late = enemies_for_layer(25);
    assert!(late.len() > early.len());
}
