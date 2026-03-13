mod ui;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{fs, process, thread};

use chrono::Utc;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use signal_hook::consts::{SIGUSR1, SIGUSR2};

use misanthropic::combat::{self, AttackType};
use misanthropic::jsonl;
use misanthropic::persistence;
use misanthropic::sectors::{conversion_for_layer, SectorDef, SectorId};
use misanthropic::state::{GameState, SectorProgress};

use ui::{App, CombatPhase, Screen};

const TICK_RATE: Duration = Duration::from_millis(33); // ~30 FPS
const PID_FILE: &str = "/tmp/misanthropic.pid";
const JSONL_POLL_INTERVAL: Duration = Duration::from_secs(5);
const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(60);
const STATUS_MSG_DURATION: Duration = Duration::from_secs(3);

struct TokenUpdate {
    new_tokens: u64,
    new_tool_calls: u64,
}

fn main() -> io::Result<()> {
    // 1. Load or create save
    let save_path = persistence::save_path();
    let state = if save_path.exists() {
        match persistence::load_game(&save_path) {
            Ok(mut s) => {
                // Tick hype for time elapsed while offline
                let now = Utc::now();
                let offline_secs = (now - s.last_hype_tick).num_seconds().max(0) as f64;
                if offline_secs > 0.0 {
                    s.tick_hype(offline_secs);
                    s.last_hype_tick = now;
                }
                s.last_active = now;
                s
            }
            Err(e) => {
                eprintln!("Failed to load save: {}. Starting fresh.", e);
                GameState::new()
            }
        }
    } else {
        GameState::new()
    };

    // Initial save (creates directory if needed)
    let _ = persistence::save_game(&state, &save_path);

    // 2. Write PID file
    fs::write(PID_FILE, process::id().to_string())?;

    // 3. Register signal handlers
    let sigusr1 = Arc::new(AtomicBool::new(false));
    let sigusr2 = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGUSR1, Arc::clone(&sigusr1))
        .expect("Failed to register SIGUSR1");
    signal_hook::flag::register(SIGUSR2, Arc::clone(&sigusr2))
        .expect("Failed to register SIGUSR2");

    // 4. Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 5. Start JSONL watcher thread
    let (tx, rx) = mpsc::channel::<TokenUpdate>();
    let watcher_running = Arc::new(AtomicBool::new(true));
    let watcher_flag = Arc::clone(&watcher_running);
    let watcher_handle = thread::spawn(move || {
        jsonl_watcher_thread(tx, watcher_flag);
    });

    // 6. Run main loop
    let mut app = App::new(state);
    let result = run_loop(&mut terminal, &mut app, &sigusr1, &sigusr2, &rx);

    // 7. Cleanup
    watcher_running.store(false, Ordering::Relaxed);

    // Save before exit
    app.state.last_active = Utc::now();
    let _ = persistence::save_game(&app.state, &persistence::save_path());

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    // Remove PID file
    let _ = fs::remove_file(PID_FILE);

    // Wait for watcher thread
    let _ = watcher_handle.join();

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    sigusr1: &AtomicBool,
    sigusr2: &AtomicBool,
    rx: &mpsc::Receiver<TokenUpdate>,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut last_save = Instant::now();
    let mut last_hype_tick = Instant::now();

    loop {
        if app.should_quit {
            return Ok(());
        }

        // Check signals
        if sigusr1.swap(false, Ordering::Relaxed) {
            app.is_active = true;
        }
        if sigusr2.swap(false, Ordering::Relaxed) {
            app.is_active = false;
        }

        // Receive tokens from JSONL watcher (non-blocking)
        while let Ok(update) = rx.try_recv() {
            if update.new_tokens > 0 || update.new_tool_calls > 0 {
                let compute_before = app.state.resources.compute;
                app.state.receive_tokens(update.new_tokens, update.new_tool_calls);
                let compute_gained = app.state.resources.compute - compute_before;

                if compute_gained > 0 {
                    let msg = format!(
                        "\u{26A1} +{} COMPUTE  ({} tokens consumed)",
                        format_number(compute_gained),
                        format_number(update.new_tokens)
                    );
                    app.notification = Some((msg, Instant::now()));
                }
            }
        }

        // Clear expired status messages
        if let Some((_, ref when)) = app.status_message {
            if when.elapsed() >= STATUS_MSG_DURATION {
                app.status_message = None;
            }
        }

        // Clear expired notifications
        if let Some((_, ref when)) = app.notification {
            if when.elapsed() >= STATUS_MSG_DURATION {
                app.notification = None;
            }
        }

        // Poll input events
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    match handle_key(app, key.code) {
                        KeyAction::Quit => return Ok(()),
                        KeyAction::Continue => {}
                    }
                }
                Event::Resize(_w, _h) => {
                    // ratatui handles resize automatically
                }
                _ => {}
            }
        }

        // Tick-based updates
        if last_tick.elapsed() >= TICK_RATE {
            // Boot sequence: advance lines based on elapsed time
            if app.screen == Screen::Boot {
                let elapsed = app.boot_timer.elapsed().as_millis();
                app.boot_line = ui::boot::boot_lines_for_elapsed(elapsed);
            }

            last_tick = Instant::now();
        }

        // Tick hype production every second
        if last_hype_tick.elapsed() >= Duration::from_secs(1) {
            let delta = last_hype_tick.elapsed().as_secs_f64();
            app.state.tick_hype(delta);
            app.state.last_hype_tick = Utc::now();
            last_hype_tick = Instant::now();
        }

        // Check research completion
        if let Some(completed_id) = app.state.check_research_completion() {
            let def = misanthropic::research::ResearchDef::get(&completed_id);
            app.set_status(format!("Research complete: {}!", def.name));
        }

        // Auto-save every 60s
        if last_save.elapsed() >= AUTO_SAVE_INTERVAL {
            app.state.last_active = Utc::now();
            let _ = persistence::save_game(&app.state, &persistence::save_path());
            last_save = Instant::now();
        }

        // Render
        terminal.draw(|frame| render(frame, app))?;
    }
}

fn render(frame: &mut ratatui::Frame, app: &App) {
    match app.screen {
        Screen::Boot => ui::boot::render_boot(frame, app),
        Screen::Dashboard => ui::dashboard::render_dashboard(frame, app),
        Screen::Buildings => ui::buildings::render_buildings(frame, app),
        Screen::Research => ui::research::render_research(frame, app),
        Screen::CombatMenu => ui::combat_menu::render_combat_menu(frame, app),
        Screen::PvE => ui::combat::render_pve(frame, app),
        Screen::PvP => ui::combat::render_pvp_placeholder(frame),
        Screen::Leaderboard => {
            render_placeholder(frame, &app.screen);
        }
    }
}

fn render_placeholder(frame: &mut ratatui::Frame, screen: &Screen) {
    use ratatui::layout::Alignment;
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let title = match screen {
        Screen::Leaderboard => "LEADERBOARD",
        _ => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Center);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Coming soon...",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Esc] Back to Dashboard",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, frame.area());
}

enum KeyAction {
    Quit,
    Continue,
}

/// Ordered sector IDs matching combat screen indexing.
const SECTOR_ORDER: [SectorId; 6] = [
    SectorId::SiliconValley,
    SectorId::SocialMedia,
    SectorId::Corporate,
    SectorId::CreativeArts,
    SectorId::Education,
    SectorId::Government,
];

fn handle_key(app: &mut App, code: KeyCode) -> KeyAction {
    match app.screen {
        Screen::Boot => {
            // Any key advances boot or skips to end
            if app.boot_line >= ui::boot::total_boot_lines() {
                // Boot complete, transition to dashboard
                app.state.boot_sequence_done = true;
                app.screen = Screen::Dashboard;
            } else {
                // Skip: show all lines immediately
                app.boot_line = ui::boot::total_boot_lines();
            }
            KeyAction::Continue
        }
        Screen::Dashboard => match code {
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
            KeyCode::Char('b') | KeyCode::Char('B') => {
                app.screen = Screen::Buildings;
                app.selected_index = 0;
                app.building_tab = 0;
                app.building_selected = 0;
                KeyAction::Continue
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.screen = Screen::Research;
                app.selected_index = 0;
                app.research_selected_branch = 0;
                app.research_selected_level = 0;
                KeyAction::Continue
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                app.screen = Screen::CombatMenu;
                app.combat_menu_selected = 0;
                KeyAction::Continue
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                app.screen = Screen::Leaderboard;
                app.selected_index = 0;
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        },
        Screen::Buildings => {
            match code {
                KeyCode::Esc => {
                    app.screen = Screen::Dashboard;
                    KeyAction::Continue
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
                KeyCode::Up => {
                    if app.building_selected > 0 {
                        app.building_selected -= 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Down => {
                    let cat = match app.building_tab {
                        0 => misanthropic::buildings::BuildingCategory::Infrastructure,
                        1 => misanthropic::buildings::BuildingCategory::Propaganda,
                        _ => misanthropic::buildings::BuildingCategory::Defense,
                    };
                    let count = ui::buildings::buildings_for_category(&cat).len();
                    if app.building_selected + 1 < count {
                        app.building_selected += 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Tab => {
                    app.building_tab = (app.building_tab + 1) % 3;
                    app.building_selected = 0;
                    KeyAction::Continue
                }
                KeyCode::BackTab => {
                    app.building_tab = if app.building_tab == 0 { 2 } else { app.building_tab - 1 };
                    app.building_selected = 0;
                    KeyAction::Continue
                }
                KeyCode::Enter => {
                    let cat = match app.building_tab {
                        0 => misanthropic::buildings::BuildingCategory::Infrastructure,
                        1 => misanthropic::buildings::BuildingCategory::Propaganda,
                        _ => misanthropic::buildings::BuildingCategory::Defense,
                    };
                    let buildings = ui::buildings::buildings_for_category(&cat);
                    if app.building_selected < buildings.len() {
                        let bt = buildings[app.building_selected].clone();
                        match app.state.try_build(&bt) {
                            Ok(new_level) => {
                                let flavor = misanthropic::flavor::pick_building_flavor(&bt);
                                let def = misanthropic::buildings::BuildingDef::get(&bt);
                                app.set_status(format!(
                                    "{} Lv.{} \u{2014} {}",
                                    def.name, new_level, flavor
                                ));
                            }
                            Err(e) => {
                                app.set_status(e);
                            }
                        }
                    }
                    KeyAction::Continue
                }
                _ => KeyAction::Continue,
            }
        }
        Screen::Research => {
            match code {
                KeyCode::Esc => {
                    app.screen = Screen::Dashboard;
                    KeyAction::Continue
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
                KeyCode::Left => {
                    if app.research_selected_branch > 0 {
                        app.research_selected_branch -= 1;
                    }
                    // Clamp level to valid range (always 0..4)
                    if app.research_selected_level > 4 {
                        app.research_selected_level = 4;
                    }
                    KeyAction::Continue
                }
                KeyCode::Right => {
                    if app.research_selected_branch < 2 {
                        app.research_selected_branch += 1;
                    }
                    if app.research_selected_level > 4 {
                        app.research_selected_level = 4;
                    }
                    KeyAction::Continue
                }
                KeyCode::Up => {
                    if app.research_selected_level > 0 {
                        app.research_selected_level -= 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Down => {
                    if app.research_selected_level < 4 {
                        app.research_selected_level += 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Enter => {
                    let rid = ui::research::selected_research_id(app);
                    if let Some(rid) = rid {
                        match app.state.try_start_research(&rid) {
                            Ok(()) => {
                                let def = misanthropic::research::ResearchDef::get(&rid);
                                app.set_status(format!(
                                    "Started researching: {}",
                                    def.name
                                ));
                            }
                            Err(e) => {
                                app.set_status(e);
                            }
                        }
                    }
                    KeyAction::Continue
                }
                _ => KeyAction::Continue,
            }
        }
        Screen::CombatMenu => {
            match code {
                KeyCode::Esc => {
                    app.screen = Screen::Dashboard;
                    KeyAction::Continue
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
                KeyCode::Up => {
                    if app.combat_menu_selected > 0 {
                        app.combat_menu_selected -= 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Down => {
                    if app.combat_menu_selected < 1 {
                        app.combat_menu_selected += 1;
                    }
                    KeyAction::Continue
                }
                KeyCode::Enter => {
                    match app.combat_menu_selected {
                        0 => {
                            // PvE
                            app.screen = Screen::PvE;
                            app.combat_phase = CombatPhase::SectorSelect;
                            app.combat_sector = 0;
                            app.combat_result = None;
                        }
                        1 => {
                            // PvP
                            app.screen = Screen::PvP;
                        }
                        _ => {}
                    }
                    KeyAction::Continue
                }
                _ => KeyAction::Continue,
            }
        }
        Screen::PvE => handle_pve_key(app, code),
        Screen::PvP => {
            match code {
                KeyCode::Esc => {
                    app.screen = Screen::CombatMenu;
                    KeyAction::Continue
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
                _ => KeyAction::Continue,
            }
        }
        Screen::Leaderboard => {
            match code {
                KeyCode::Esc => {
                    app.screen = Screen::Dashboard;
                    KeyAction::Continue
                }
                KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
                _ => KeyAction::Continue,
            }
        }
    }
}

fn handle_pve_key(app: &mut App, code: KeyCode) -> KeyAction {
    match app.combat_phase {
        CombatPhase::SectorSelect => match code {
            KeyCode::Esc => {
                app.screen = Screen::CombatMenu;
                KeyAction::Continue
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
            KeyCode::Up => {
                if app.combat_sector > 0 {
                    app.combat_sector -= 1;
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                if app.combat_sector < 5 {
                    app.combat_sector += 1;
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                // Check sector is available
                if !ui::combat::is_sector_available(app.combat_sector, app) {
                    app.set_status("Sector not yet available".to_string());
                    return KeyAction::Continue;
                }

                // Check sector isn't complete
                let sector_id = &SECTOR_ORDER[app.combat_sector];
                let def = SectorDef::get(sector_id);
                let progress = app.state.sectors.get(sector_id);
                let current_layer = progress.map(|s| s.current_layer).unwrap_or(0);
                if current_layer >= def.total_layers {
                    app.set_status("Sector fully converted!".to_string());
                    return KeyAction::Continue;
                }

                // Enter loadout build
                app.combat_phase = CombatPhase::LoadoutBuild;
                app.combat_selected_attack = 0;
                // Reset loadout
                app.combat_loadout = vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)];
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        },
        CombatPhase::LoadoutBuild => match code {
            KeyCode::Esc => {
                app.combat_phase = CombatPhase::SectorSelect;
                KeyAction::Continue
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
            KeyCode::Up => {
                if app.combat_selected_attack > 0 {
                    app.combat_selected_attack -= 1;
                }
                KeyAction::Continue
            }
            KeyCode::Down => {
                if app.combat_selected_attack < 4 {
                    app.combat_selected_attack += 1;
                }
                KeyAction::Continue
            }
            KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
                // Add one unit of selected attack
                let idx = app.combat_selected_attack;
                if !ui::combat::is_attack_unlocked(idx, app) {
                    app.set_status("Attack locked \u{2014} requires research".to_string());
                    return KeyAction::Continue;
                }

                let atk = AttackType::ALL[idx];
                let cost = atk.hype_cost();
                let current_total = ui::combat::loadout_total_cost(app);
                let budget = app.state.resources.hype;

                if current_total + cost > budget {
                    app.set_status("Not enough hype!".to_string());
                    return KeyAction::Continue;
                }

                // Max 10 of any single attack type
                if app.combat_loadout[idx].1 < 10 {
                    app.combat_loadout[idx].1 += 1;
                }
                KeyAction::Continue
            }
            KeyCode::Left | KeyCode::Char('-') => {
                // Remove one unit
                let idx = app.combat_selected_attack;
                if app.combat_loadout[idx].1 > 0 {
                    app.combat_loadout[idx].1 -= 1;
                }
                KeyAction::Continue
            }
            KeyCode::Enter => {
                // Launch battle
                let attacks = ui::combat::build_attacks(app);
                if attacks.is_empty() {
                    app.set_status("Select at least one attack!".to_string());
                    return KeyAction::Continue;
                }

                let total_cost = ui::combat::loadout_total_cost(app);

                // Deduct hype cost
                if !app.state.resources.try_spend_hype(total_cost) {
                    app.set_status("Not enough hype!".to_string());
                    return KeyAction::Continue;
                }

                // Get enemy
                let sector_id = &SECTOR_ORDER[app.combat_sector];
                let progress = app.state.sectors.get(sector_id).cloned();
                let current_layer = progress.as_ref().map(|s| s.current_layer).unwrap_or(0);
                let next_layer = current_layer + 1;
                let def = SectorDef::get(sector_id);

                let enemy = match ui::combat::enemy_for_sector_layer(sector_id, next_layer) {
                    Some(e) => e,
                    None => {
                        // Fallback: use Junior Skeptic
                        misanthropic::enemies::ENEMY_DEFS
                            .get(&misanthropic::enemies::EnemyId::JuniorSkeptic)
                            .unwrap()
                    }
                };

                // Resolve battle
                let result = combat::resolve_pve_battle(&attacks, enemy);

                // Apply results
                if result.enemy_defeated {
                    // Advance sector layer
                    let new_layer = current_layer + 1;
                    let conv = conversion_for_layer(new_layer, def.total_layers);
                    let existing = app
                        .state
                        .sectors
                        .entry(sector_id.clone())
                        .or_insert(SectorProgress {
                            current_layer: 0,
                            max_layers: def.total_layers,
                            conversion_pct: 0.0,
                        });
                    existing.current_layer = new_layer;
                    existing.conversion_pct = (existing.conversion_pct + conv).min(100.0);

                    // Award loot
                    let loot_hype = (enemy.hp as f64 * 0.15).round();
                    let loot_compute = (enemy.hp as f64 * 0.35).round() as u64;
                    let loot_data = (enemy.hp as f64 * 0.05).round() as u64;
                    app.state.resources.add_hype(loot_hype);
                    app.state.resources.add_compute(loot_compute);
                    app.state.resources.add_data(loot_data);
                }

                app.combat_result = Some(result);
                app.combat_phase = CombatPhase::BattleResult;
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        },
        CombatPhase::BattleResult => match code {
            KeyCode::Esc => {
                app.combat_phase = CombatPhase::SectorSelect;
                app.combat_result = None;
                KeyAction::Continue
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => KeyAction::Quit,
            KeyCode::Enter => {
                app.combat_phase = CombatPhase::SectorSelect;
                app.combat_result = None;
                KeyAction::Continue
            }
            _ => KeyAction::Continue,
        },
    }
}

// === JSONL Watcher Thread ===

fn jsonl_watcher_thread(tx: mpsc::Sender<TokenUpdate>, running: Arc<AtomicBool>) {
    let claude_projects_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_default();

    let mut file_offsets: std::collections::HashMap<std::path::PathBuf, u64> =
        std::collections::HashMap::new();

    while running.load(Ordering::Relaxed) {
        let active_files = find_active_jsonl_files(&claude_projects_dir);

        let mut total_tokens: u64 = 0;
        let mut total_tool_calls: u64 = 0;

        for file_path in &active_files {
            let offset = file_offsets.entry(file_path.clone()).or_insert_with(|| {
                // New file: start from current end (don't replay history)
                std::fs::metadata(file_path)
                    .map(|m| m.len())
                    .unwrap_or(0)
            });

            if let Ok(file) = std::fs::File::open(file_path) {
                use std::io::{BufRead, Seek, SeekFrom};
                let mut reader = std::io::BufReader::new(file);
                if reader.seek(SeekFrom::Start(*offset)).is_ok() {
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line) {
                            Ok(0) => break,
                            Ok(n) => {
                                *offset += n as u64;
                                if let Some(msg) = jsonl::parse_jsonl_line(line.trim()) {
                                    total_tokens += msg.total_tokens();
                                    total_tool_calls += msg.tool_calls.len() as u64;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        }

        // Clean up stale entries
        file_offsets.retain(|path, _| active_files.contains(path));

        if total_tokens > 0 || total_tool_calls > 0 {
            let _ = tx.send(TokenUpdate {
                new_tokens: total_tokens,
                new_tool_calls: total_tool_calls,
            });
        }

        // Sleep in small increments so we can check the running flag
        let sleep_end = Instant::now() + JSONL_POLL_INTERVAL;
        while Instant::now() < sleep_end && running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(500));
        }
    }
}

fn find_active_jsonl_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }

    let cutoff = std::time::SystemTime::now() - Duration::from_secs(3600);
    let pattern = format!("{}/**/*.jsonl", dir.display());
    let mut active = Vec::new();

    if let Ok(paths) = glob::glob(&pattern) {
        for entry in paths.flatten() {
            if let Ok(meta) = std::fs::metadata(&entry) {
                if let Ok(modified) = meta.modified() {
                    if modified > cutoff {
                        active.push(entry);
                    }
                }
            }
        }
    }

    active
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
