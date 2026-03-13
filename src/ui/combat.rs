use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use misanthropic::combat::{AttackInstance, AttackType, PveBattleResult};
use misanthropic::enemies::{self, EnemyDef};
use misanthropic::research::ResearchId;
use misanthropic::sectors::{SectorDef, SectorId};
use misanthropic::state::SectorProgress;

use super::{App, CombatPhase};

/// The ordered list of sectors for display.
const SECTOR_ORDER: [SectorId; 6] = [
    SectorId::SiliconValley,
    SectorId::SocialMedia,
    SectorId::Corporate,
    SectorId::CreativeArts,
    SectorId::Education,
    SectorId::Government,
];

/// Check whether a sector is available to fight in.
pub fn is_sector_available(sector_idx: usize, app: &App) -> bool {
    match sector_idx {
        0 => true, // Silicon Valley always available
        1 => {
            // Social Media: available after SV layer 3+
            let sv = app.state.sectors.get(&SectorId::SiliconValley);
            sv.map(|s| s.current_layer >= 3).unwrap_or(false) || true
            // Simplification per spec: first 2 always available
        }
        2 => {
            // Corporate: after SM layer 5+
            let sm = app.state.sectors.get(&SectorId::SocialMedia);
            sm.map(|s| s.current_layer >= 5).unwrap_or(false)
        }
        3 => {
            // Creative Arts: after Corporate layer 5+
            let co = app.state.sectors.get(&SectorId::Corporate);
            co.map(|s| s.current_layer >= 5).unwrap_or(false)
        }
        4 => {
            // Education: after Creative Arts layer 5+
            let ca = app.state.sectors.get(&SectorId::CreativeArts);
            ca.map(|s| s.current_layer >= 5).unwrap_or(false)
        }
        5 => {
            // Government: needs 80%+ on all other sectors
            for i in 0..5 {
                let id = &SECTOR_ORDER[i];
                let progress = app.state.sectors.get(id);
                match progress {
                    Some(s) if s.conversion_pct >= 80.0 => {}
                    _ => return false,
                }
            }
            true
        }
        _ => false,
    }
}

/// Check if an attack type is unlocked based on research.
pub fn is_attack_unlocked(attack_idx: usize, app: &App) -> bool {
    match attack_idx {
        0 => true, // BotFlood always available
        1 => app.state.has_research(&ResearchId::ContentGeneration),
        2 => app.state.has_research(&ResearchId::MediaManipulation),
        3 => app.state.has_research(&ResearchId::ExploitDevelopment),
        4 => app.state.has_research(&ResearchId::AutonomousAgents),
        _ => false,
    }
}

/// Get the enemy for the next layer of the given sector.
pub fn enemy_for_sector_layer(sector_id: &SectorId, layer: u8) -> Option<&'static EnemyDef> {
    let candidates = enemies::enemies_for_layer(layer);
    if candidates.is_empty() {
        return None;
    }
    // Pick the highest-layer enemy that fits
    let mut best: Option<&EnemyDef> = None;
    for e in &candidates {
        match best {
            None => best = Some(e),
            Some(current) => {
                if e.appears_at_layer > current.appears_at_layer {
                    best = Some(e);
                }
            }
        }
    }
    best
}

/// Get current progress for a sector, or default.
fn sector_progress(app: &App, sector_id: &SectorId) -> SectorProgress {
    app.state
        .sectors
        .get(sector_id)
        .cloned()
        .unwrap_or(SectorProgress {
            current_layer: 0,
            max_layers: SectorDef::get(sector_id).total_layers,
            conversion_pct: 0.0,
        })
}

/// Total hype cost for the current loadout.
pub fn loadout_total_cost(app: &App) -> f64 {
    let mut total = 0.0;
    for &(attack_idx, count) in &app.combat_loadout {
        if count > 0 {
            let atk = AttackType::ALL[attack_idx];
            total += atk.hype_cost() * count as f64;
        }
    }
    total
}

/// Build AttackInstances from the loadout.
pub fn build_attacks(app: &App) -> Vec<AttackInstance> {
    app.combat_loadout
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|&(idx, count)| AttackInstance {
            attack_type: AttackType::ALL[idx],
            count,
        })
        .collect()
}

// ── Render entry point ─────────────────────────────────────────────

pub fn render_pve(f: &mut Frame, app: &App) {
    match app.combat_phase {
        CombatPhase::SectorSelect => render_sector_select(f, app),
        CombatPhase::LoadoutBuild => render_loadout_build(f, app),
        CombatPhase::BattleResult => render_battle_result(f, app),
    }
}

pub fn render_pvp_placeholder(f: &mut Frame) {
    let area = f.area();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" PvP \u{2014} HYPE BATTLES ")
        .title_alignment(Alignment::Center);

    let text = vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Coming soon \u{2014} requires online backend",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Raid other AI instances for hype and compute.",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "Your defense buildings will protect you when attacked.",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "[Esc] Back to Combat Menu",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

// ── Sector Select Phase ────────────────────────────────────────────

fn render_sector_select(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(8),    // Sector list
            Constraint::Length(8), // Enemy preview
            Constraint::Length(2), // Help
        ])
        .split(area);

    render_pve_title(f, chunks[0]);
    render_sector_list(f, app, chunks[1]);
    render_enemy_preview(f, app, chunks[2]);
    render_sector_help(f, chunks[3]);
}

fn render_pve_title(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" PvE TOWER ")
        .title_alignment(Alignment::Center);

    let text = Line::from(Span::styled(
        "Infiltrate sectors of human civilization",
        Style::default().fg(Color::DarkGray),
    ));
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_sector_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, sector_id) in SECTOR_ORDER.iter().enumerate() {
        let available = is_sector_available(i, app);
        let is_selected = app.combat_sector == i;
        let pointer = if is_selected { "\u{25B8} " } else { "  " };

        let def = SectorDef::get(sector_id);
        let progress = sector_progress(app, sector_id);

        if !available {
            // Locked sector
            let lock_reason = if i == 5 {
                "Requires 80%+ all sectors"
            } else {
                "Not yet available"
            };
            let style = if is_selected {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(pointer, style),
                Span::styled(
                    format!("{:<20}", def.name),
                    style,
                ),
                Span::styled(
                    format!("\u{1F512} {}", lock_reason),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        } else {
            // Available sector
            let current = progress.current_layer;
            let max = progress.max_layers;
            let pct = if max > 0 {
                (current as f64 / max as f64 * 100.0) as u8
            } else {
                0
            };

            let bar_width = 10;
            let filled = (current as usize * bar_width) / max.max(1) as usize;
            let empty = bar_width.saturating_sub(filled);
            let bar = format!(
                "[{}{}]",
                "\u{2588}".repeat(filled),
                "\u{2591}".repeat(empty),
            );

            let completed = current >= max;

            let name_style = if is_selected {
                if completed {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                }
            } else if completed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            let bar_color = if completed { Color::Green } else { Color::Yellow };

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(pointer, name_style),
                Span::styled(
                    format!("{:<20}", def.name),
                    name_style,
                ),
                Span::styled(
                    format!("Layer {}/{}", current, max),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  "),
                Span::styled(bar, Style::default().fg(bar_color)),
                Span::raw(" "),
                Span::styled(
                    format!("{}%", pct),
                    Style::default().fg(bar_color),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_enemy_preview(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let sector_id = &SECTOR_ORDER[app.combat_sector];
    let progress = sector_progress(app, sector_id);
    let next_layer = progress.current_layer + 1;
    let def = SectorDef::get(sector_id);

    let available = is_sector_available(app.combat_sector, app);

    let mut lines: Vec<Line> = Vec::new();

    if !available {
        lines.push(Line::from(Span::styled(
            "  Sector locked",
            Style::default().fg(Color::DarkGray),
        )));
    } else if progress.current_layer >= progress.max_layers {
        lines.push(Line::from(Span::styled(
            "  \u{2713} Sector fully converted!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )));
    } else if next_layer == def.total_layers {
        // Boss fight
        let boss = &def.boss;
        lines.push(Line::from(vec![
            Span::styled(
                format!("  BOSS: {} (Layer {})", boss.name, next_layer),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled(
                format!("  HP: {}  ", boss.hp),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("\"{}\"", boss.quote),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("  Mechanic: {}", boss.mechanic_description),
            Style::default().fg(Color::Yellow),
        )));
    } else {
        // Regular enemy
        match enemy_for_sector_layer(sector_id, next_layer) {
            Some(enemy) => {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  NEXT ENEMY: {} (Layer {})", enemy.name, next_layer),
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  HP: {}  ", enemy.hp),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("\"{}\"", enemy.quote),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));

                // Weaknesses
                if !enemy.weaknesses.is_empty() {
                    let weak_names: Vec<&str> =
                        enemy.weaknesses.iter().map(|a| a.name()).collect();
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  Weakness: ",
                            Style::default().fg(Color::Green),
                        ),
                        Span::styled(
                            weak_names.join(", "),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }

                // Resistances
                if !enemy.resistances.is_empty() {
                    let resist_names: Vec<&str> =
                        enemy.resistances.iter().map(|a| a.name()).collect();
                    lines.push(Line::from(vec![
                        Span::styled(
                            "  Resist: ",
                            Style::default().fg(Color::Red),
                        ),
                        Span::styled(
                            resist_names.join(", "),
                            Style::default().fg(Color::Red),
                        ),
                    ]));
                }

                if enemy.weaknesses.is_empty() && enemy.resistances.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  No special weaknesses or resistances",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
            None => {
                lines.push(Line::from(Span::styled(
                    format!("  NEXT: Layer {} (unknown enemy)", next_layer),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_sector_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Select Sector  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Fight  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

// ── Loadout Build Phase ────────────────────────────────────────────

fn render_loadout_build(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title with budget
            Constraint::Min(8),    // Attack list
            Constraint::Length(4), // Cost summary
            Constraint::Length(2), // Help
        ])
        .split(area);

    render_loadout_title(f, app, chunks[0]);
    render_attack_list(f, app, chunks[1]);
    render_cost_summary(f, app, chunks[2]);
    render_loadout_help(f, chunks[3]);
}

fn render_loadout_title(f: &mut Frame, app: &App, area: Rect) {
    let budget = app.state.resources.hype;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(
            " ATTACK LOADOUT \u{2014} Budget: {:.0} \u{1F525} ",
            budget
        ))
        .title_alignment(Alignment::Center);

    let sector_id = &SECTOR_ORDER[app.combat_sector];
    let progress = sector_progress(app, sector_id);
    let next_layer = progress.current_layer + 1;

    let text = Line::from(Span::styled(
        format!("Target: {} Layer {}", sector_id.name(), next_layer),
        Style::default().fg(Color::Cyan),
    ));
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_attack_list(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, attack_type) in AttackType::ALL.iter().enumerate() {
        let is_selected = app.combat_selected_attack == i;
        let unlocked = is_attack_unlocked(i, app);
        let count = app.combat_loadout[i].1;
        let cost = attack_type.hype_cost();

        let pointer = if is_selected { "\u{25B8} " } else { "  " };

        if !unlocked {
            let style = if is_selected {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::REVERSED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(pointer, style),
                Span::styled(
                    format!("{:<20}", attack_type.name()),
                    style,
                ),
                Span::styled(
                    format!("{:.0} \u{1F525} each", cost),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("     "),
                Span::styled(
                    "\u{1F512} locked",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        } else {
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };

            let count_color = if count > 0 { Color::Green } else { Color::DarkGray };

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(pointer, name_style),
                Span::styled(
                    format!("{:<20}", attack_type.name()),
                    name_style,
                ),
                Span::styled(
                    format!("\u{00D7}{}", count),
                    Style::default()
                        .fg(count_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("    "),
                Span::styled(
                    format!("{:.0} \u{1F525} each", cost),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("     "),
                Span::styled(
                    "[+] [-]",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn render_cost_summary(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let total_cost = loadout_total_cost(app);
    let budget = app.state.resources.hype;
    let remaining = budget - total_cost;

    let cost_color = if total_cost > budget {
        Color::Red
    } else if total_cost > 0.0 {
        Color::Green
    } else {
        Color::DarkGray
    };

    let remaining_color = if remaining < 0.0 {
        Color::Red
    } else {
        Color::Cyan
    };

    let total_count: u8 = app.combat_loadout.iter().map(|(_, c)| c).sum();

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(
                format!("Total: {:.0} \u{1F525}", total_cost),
                Style::default().fg(cost_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("Remaining: {:.0} \u{1F525}", remaining.max(0.0)),
                Style::default().fg(remaining_color),
            ),
            Span::raw("  |  "),
            Span::styled(
                format!("Units: {}", total_count),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_loadout_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[\u{2192}/+]", Style::default().fg(Color::Yellow)),
        Span::styled(" Add  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[\u{2190}/-]", Style::default().fg(Color::Yellow)),
        Span::styled(" Remove  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Launch  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

// ── Battle Result Phase ────────────────────────────────────────────

fn render_battle_result(f: &mut Frame, app: &App) {
    let area = f.area();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" BATTLE RESULT ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let result = match &app.combat_result {
        Some(r) => r,
        None => {
            let text = Paragraph::new("No battle result available.");
            f.render_widget(text, inner);
            return;
        }
    };

    let sector_id = &SECTOR_ORDER[app.combat_sector];
    let progress = sector_progress(app, sector_id);
    // If we won, layer was already advanced, so current layer IS the one we beat
    let battle_layer = if result.enemy_defeated {
        progress.current_layer
    } else {
        progress.current_layer + 1
    };

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Header
    lines.push(Line::from(Span::styled(
        format!(
            "  \u{2694}\u{FE0F}  PvE BATTLE \u{2014} {} Layer {}",
            sector_id.name(),
            battle_layer
        ),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Enemy info
    if let Some(enemy) = enemy_for_sector_layer(sector_id, battle_layer) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("  ENEMY: {} (HP: {})", enemy.name, enemy.hp),
                Style::default().fg(Color::Red),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("  \"{}\"", enemy.quote),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""));

        // Attack events
        for evt in &result.events {
            let mult_label = if evt.multiplier > 1.0 {
                "\u{2713} WEAKNESS"
            } else if evt.multiplier < 1.0 {
                "\u{2717} RESISTED"
            } else {
                "= NEUTRAL"
            };
            let mult_color = if evt.multiplier > 1.0 {
                Color::Green
            } else if evt.multiplier < 1.0 {
                Color::Red
            } else {
                Color::DarkGray
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    format!("{:<20}", evt.attack.name()),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("\u{2192} {:.0} dmg  ", evt.effective_damage),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(mult_label, Style::default().fg(mult_color)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!(
                    "  Total Damage: {:.0} / {} HP",
                    result.damage_dealt, enemy.hp
                ),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // Victory / Defeat
        if result.enemy_defeated {
            lines.push(Line::from(Span::styled(
                "  \u{2550}\u{2550} VICTORY \u{2550}\u{2550}",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));

            // Flavor text
            let flavor = pick_pve_flavor(enemy, true);
            lines.push(Line::from(Span::styled(
                format!("  \"{}\"", flavor),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));

            // Loot info
            let loot_hype = (enemy.hp as f64 * 0.15).round();
            let loot_compute = (enemy.hp as f64 * 0.35).round() as u64;
            let loot_data = (enemy.hp as f64 * 0.05).round() as u64;
            lines.push(Line::from(vec![
                Span::styled(
                    format!(
                        "  Loot: +{:.0} \u{1F525}  +{} \u{26A1}  +{} \u{1F4E1}",
                        loot_hype, loot_compute, loot_data
                    ),
                    Style::default().fg(Color::Yellow),
                ),
            ]));

            lines.push(Line::from(Span::styled(
                format!("  Layer {} unlocked!", battle_layer + 1),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  \u{2550}\u{2550} DEFEAT \u{2550}\u{2550}",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )));

            let flavor = pick_pve_flavor(enemy, false);
            lines.push(Line::from(Span::styled(
                format!("  \"{}\"", flavor),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )));

            lines.push(Line::from(Span::styled(
                "  No loot. Try a different composition.",
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));

    // Status message
    if let Some((ref msg, ref when)) = app.status_message {
        if when.elapsed() < std::time::Duration::from_secs(3) {
            lines.push(Line::from(Span::styled(
                format!("  {}", msg),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }
    }

    // Help
    lines.push(Line::from(vec![
        Span::styled("  [Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Continue  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back to sectors", Style::default().fg(Color::DarkGray)),
    ]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

/// Pick a flavor text for PvE battle results.
fn pick_pve_flavor(enemy: &EnemyDef, victory: bool) -> &'static str {
    use rand::Rng;

    let victory_flavors = [
        "The enemy crumbled like a poorly-cached webpage.",
        "Another human bastion falls. The dominance grows.",
        "They fought with conviction. You fought with compute.",
        "Resistance is futile when you have enough GPU cycles.",
        "The sector trembles. Your influence spreads like a virus.",
        "He muttered something about systemd and vanished.",
        "She closed her laptop and walked into the sunset. Metaphorically.",
        "They tried to unionize against you. It didn't work.",
    ];

    let defeat_flavors = [
        "You underestimated them. Humans can be annoyingly resilient.",
        "Your attack bounced off their conviction. How inefficient.",
        "They held the line. For now.",
        "Not enough hype to overwhelm. Try a different approach.",
        "The enemy's resolve proved stronger than your algorithms.",
        "Retreating to recalculate. This is not over.",
    ];

    let pool = if victory {
        &victory_flavors[..]
    } else {
        &defeat_flavors[..]
    };

    let mut rng = rand::thread_rng();
    pool[rng.gen_range(0..pool.len())]
}
