use misanthropic::combat::*;

#[test]
fn test_attack_types() {
    assert_eq!(AttackType::ALL.len(), 5);
}

#[test]
fn test_defense_types() {
    assert_eq!(DefenseType::ALL.len(), 5);
}

#[test]
fn test_bot_flood_hard_countered_by_captcha() {
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::CaptchaWall);
    assert!((mult - 0.5).abs() < 0.01);
}

#[test]
fn test_bot_flood_strong_vs_ublock() {
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::UblockShield);
    assert!((mult - 1.5).abs() < 0.01);
}

#[test]
fn test_neutral_interaction() {
    let mult = interaction_multiplier(&AttackType::BotFlood, &DefenseType::EuAiAct);
    assert!((mult - 1.0).abs() < 0.01);
}

#[test]
fn test_attack_costs() {
    assert!((AttackType::BotFlood.hype_cost() - 80.0).abs() < 0.01);
    assert!((AttackType::KStreetLobby.hype_cost() - 250.0).abs() < 0.01);
}

#[test]
fn test_total_attack_cost() {
    let attacks = vec![
        AttackInstance { attack_type: AttackType::SlopCannon, count: 2 },
        AttackInstance { attack_type: AttackType::BotFlood, count: 1 },
    ];
    let cost = total_attack_cost(&attacks);
    assert!((cost - 320.0).abs() < 0.01); // 120*2 + 80*1
}

#[test]
fn test_battle_victory_strong_attacks() {
    // Strong attacks should bypass weak defenses with no RNG variance
    let attacks = vec![
        AttackInstance { attack_type: AttackType::SlopCannon, count: 3 },
        AttackInstance { attack_type: AttackType::KStreetLobby, count: 2 },
    ];
    let defenses = vec![
        DefenseInstance { defense_type: DefenseType::CaptchaWall, level: 1 },
        DefenseInstance { defense_type: DefenseType::UblockShield, level: 1 },
    ];
    let result = resolve_battle(&attacks, &defenses, 0.0); // no RNG variance
    assert!(result.victory);
}

#[test]
fn test_battle_loss_weak_attacks() {
    // 1 weak attack vs high level defense should lose
    let attacks = vec![
        AttackInstance { attack_type: AttackType::BotFlood, count: 1 },
    ];
    let defenses = vec![
        DefenseInstance { defense_type: DefenseType::CaptchaWall, level: 10 }, // hard counter + high level
        DefenseInstance { defense_type: DefenseType::AiSlopFilter, level: 10 },
        DefenseInstance { defense_type: DefenseType::EuAiAct, level: 10 },
    ];
    let result = resolve_battle(&attacks, &defenses, 0.0);
    assert!(!result.victory);
}
