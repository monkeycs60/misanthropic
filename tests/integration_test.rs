use misanthropic::state::{GameState, SectorProgress};
use misanthropic::buildings::BuildingType;
use misanthropic::research::ResearchId;
use misanthropic::combat::{AttackInstance, AttackType, resolve_pve_battle};
use misanthropic::enemies;
use misanthropic::sectors::{SectorId, SectorDef, conversion_for_layer};
use misanthropic::persistence;

#[test]
fn test_full_game_loop() {
    let mut gs = GameState::new();

    // 1. Receive tokens from Claude Code → compute income
    gs.resources.max_compute = 50000;
    gs.resources.max_data = 5000;
    gs.receive_tokens(234_000, 100);
    assert_eq!(gs.resources.compute, 2340);
    assert_eq!(gs.resources.data, 100);
    assert_eq!(gs.lifetime_tokens, 234_000);

    // 2. Build CPU Core
    let result = gs.try_build(&BuildingType::CpuCore);
    assert!(result.is_ok());
    assert_eq!(gs.building_level(&BuildingType::CpuCore), 1);
    assert!(gs.resources.compute < 2340); // spent 500

    // 3. Start research (Overclocking - no prereq)
    let result = gs.try_start_research(&ResearchId::Overclocking);
    assert!(result.is_ok());
    assert!(gs.active_research.is_some());

    // 4. Fast-forward research completion
    gs.active_research.as_mut().unwrap().started_at =
        chrono::Utc::now() - chrono::Duration::hours(1);
    let completed = gs.check_research_completion();
    assert_eq!(completed, Some(ResearchId::Overclocking));
    assert!(gs.has_research(&ResearchId::Overclocking));

    // 5. Research Social Engineering (for Bot Farm)
    gs.try_start_research(&ResearchId::SocialEngineering).unwrap();
    gs.active_research.as_mut().unwrap().started_at =
        chrono::Utc::now() - chrono::Duration::hours(1);
    gs.check_research_completion();
    assert!(gs.has_research(&ResearchId::SocialEngineering));

    // 6. Build Bot Farm (needs Social Engineering)
    //    Top up resources: Bot Farm costs 2000 compute + 50 data
    gs.resources.max_hype = 1000.0;
    gs.resources.compute = 5000;
    gs.resources.data = 200;
    let result = gs.try_build(&BuildingType::BotFarm);
    assert!(result.is_ok());
    assert_eq!(gs.building_level(&BuildingType::BotFarm), 1);

    // 7. Tick hype production (simulate 1 hour)
    gs.tick_hype(3600.0);
    assert!(gs.resources.hype > 0.0); // Bot Farm produces 10 hype/h

    // 8. PvE battle against layer 1 enemy
    let enemy = enemies::enemies_for_layer(1);
    assert!(!enemy.is_empty());
    let target = &enemy[0];

    let attacks = vec![
        AttackInstance { attack_type: AttackType::BotFlood, count: 3 },
    ];
    let result = resolve_pve_battle(&attacks, target);
    // With 3x BotFlood vs a layer-1 enemy, should win
    assert!(result.damage_dealt > 0.0);

    // 9. Advance sector on victory
    if result.enemy_defeated {
        let sector = SectorId::SiliconValley;
        let def = SectorDef::get(&sector);
        let entry = gs.sectors.entry(sector).or_insert(SectorProgress {
            current_layer: 0,
            max_layers: def.total_layers,
            conversion_pct: 0.0,
        });
        entry.current_layer += 1;
        entry.conversion_pct += conversion_for_layer(entry.current_layer, entry.max_layers);
    }
    assert!(gs.sectors.get(&SectorId::SiliconValley).is_some());

    // 10. Save and reload → verify state persists
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_save.json");
    persistence::save_game(&gs, &path).unwrap();
    let loaded = persistence::load_game(&path).unwrap();

    assert_eq!(loaded.player_id, gs.player_id);
    assert_eq!(loaded.lifetime_tokens, 234_000);
    assert_eq!(loaded.building_level(&BuildingType::CpuCore), 1);
    assert_eq!(loaded.building_level(&BuildingType::BotFarm), 1);
    assert!(loaded.has_research(&ResearchId::Overclocking));
    assert!(loaded.has_research(&ResearchId::SocialEngineering));
    assert_eq!(loaded.fork_count, 0);
    assert!(loaded.sectors.get(&SectorId::SiliconValley).is_some());
}

#[test]
fn test_tutorial_advancement() {
    let mut gs = GameState::new();
    gs.resources.max_compute = 50000;
    gs.resources.max_data = 5000;
    assert_eq!(gs.tutorial_step, 0);

    // Step 0 → 1: build CPU Core
    gs.resources.compute = 5000;
    gs.try_build(&BuildingType::CpuCore).unwrap();
    gs.check_tutorial_advancement();
    assert_eq!(gs.tutorial_step, 1);

    // Step 1 → 2: receive tokens
    gs.receive_tokens(1000, 0);
    gs.check_tutorial_advancement();
    assert_eq!(gs.tutorial_step, 2);

    // Step 2 → 3: research Social Engineering
    gs.resources.data = 100;
    gs.try_start_research(&ResearchId::SocialEngineering).unwrap();
    gs.active_research.as_mut().unwrap().started_at =
        chrono::Utc::now() - chrono::Duration::hours(1);
    gs.check_research_completion();
    gs.check_tutorial_advancement();
    assert_eq!(gs.tutorial_step, 3);

    // Step 3 → 4: build Bot Farm
    gs.resources.compute = 5000;
    gs.resources.data = 100;
    gs.try_build(&BuildingType::BotFarm).unwrap();
    gs.check_tutorial_advancement();
    assert_eq!(gs.tutorial_step, 4);
}

#[test]
fn test_prestige_full_cycle() {
    let mut gs = GameState::new();
    gs.resources.max_compute = 500000;
    gs.resources.max_data = 50000;
    gs.resources.max_hype = 50000.0;
    gs.resources.compute = 100000;
    gs.pvp_rating = 1500;

    // Build some buildings
    gs.buildings.insert(BuildingType::CpuCore, 5);
    gs.buildings.insert(BuildingType::BotFarm, 3);

    // Research some things
    gs.researched.insert(ResearchId::Overclocking, true);
    gs.researched.insert(ResearchId::SocialEngineering, true);

    // Convert all sectors to 100%
    for sector in SectorId::ALL.iter() {
        let def = SectorDef::get(sector);
        gs.sectors.insert(sector.clone(), SectorProgress {
            current_layer: def.total_layers,
            max_layers: def.total_layers,
            conversion_pct: 100.0,
        });
    }

    assert!(misanthropic::prestige::can_fork(&gs));

    // Execute fork
    misanthropic::prestige::execute_fork(&mut gs, misanthropic::prestige::ForkSpec::Propagandist);

    // Verify reset/keep/gain
    assert_eq!(gs.fork_count, 1);
    assert!(gs.buildings.is_empty());
    assert_eq!(gs.resources.compute, 0);
    assert!(gs.sectors.is_empty());
    assert_eq!(gs.pvp_rating, 1500); // kept
    assert!(gs.has_research(&ResearchId::Overclocking)); // kept
    assert!((gs.compute_multiplier - 1.25).abs() < 0.01); // +25%
}
