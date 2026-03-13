use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use misanthropic::buildings::{BuildingCategory, BuildingDef, BuildingType, BUILDING_DEFS};
use misanthropic::research::ResearchDef;

use super::App;

/// Display order for each category.
pub fn buildings_for_category(cat: &BuildingCategory) -> Vec<&'static BuildingType> {
    use BuildingType::*;
    match cat {
        BuildingCategory::Infrastructure => vec![
            &CpuCore, &RamBank, &GpuRig, &GpuCluster, &Datacenter, &QuantumCore,
        ],
        BuildingCategory::Propaganda => vec![
            &BotFarm, &ContentMill, &MemeLab, &DeepfakeStudio, &VibeAcademy,
            &NsfwGenerator, &LobbyOffice,
        ],
        BuildingCategory::Defense => vec![
            &CaptchaWall, &AiSlopFilter, &UblockShield, &HarvardStudy, &EuAiAct,
        ],
    }
}

fn category_from_tab(tab: u8) -> BuildingCategory {
    match tab {
        0 => BuildingCategory::Infrastructure,
        1 => BuildingCategory::Propaganda,
        _ => BuildingCategory::Defense,
    }
}

pub fn render_buildings(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tab bar
            Constraint::Min(8),    // Building list
            Constraint::Length(7), // Detail panel
            Constraint::Length(3), // Resource bar
            Constraint::Length(2), // Help line
        ])
        .split(area);

    render_tab_bar(f, app, chunks[0]);
    render_building_list(f, app, chunks[1]);
    render_detail_panel(f, app, chunks[2]);
    render_resource_bar(f, app, chunks[3]);
    render_help(f, chunks[4]);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tabs = ["Infrastructure", "Propaganda", "Defense"];

    let mut spans = Vec::new();
    spans.push(Span::raw("  "));
    for (i, label) in tabs.iter().enumerate() {
        if i == app.building_tab as usize {
            spans.push(Span::styled(
                format!("[{}]", label),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(Color::DarkGray),
            ));
        }
        if i < tabs.len() - 1 {
            spans.push(Span::raw("  "));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" BUILDINGS ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(paragraph, area);
}

fn render_building_list(f: &mut Frame, app: &App, area: Rect) {
    let cat = category_from_tab(app.building_tab);
    let buildings = buildings_for_category(&cat);

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, bt) in buildings.iter().enumerate() {
        let def = BuildingDef::get(bt);
        let level = app.state.building_level(bt);
        let is_selected = i == app.building_selected;
        let pointer = if is_selected { "\u{25B8} " } else { "  " };

        // Check if locked
        let locked_reason = get_lock_reason(def, &app.state);

        if let Some(reason) = locked_reason {
            // Locked building
            let line = Line::from(vec![
                Span::raw("  "),
                Span::raw(pointer),
                Span::styled(
                    format!("{:<18}", def.name),
                    if is_selected {
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::styled(
                    format!("\u{1F512} Requires: {}", reason),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            lines.push(line);
        } else {
            // Unlocked building
            let cost = def.cost_at_level(level + 1);
            let max_reached = level >= def.max_level;
            let affordable = !max_reached
                && app
                    .state
                    .resources
                    .can_afford(cost.compute, cost.data, cost.hype);

            let level_str = if level > 0 {
                format!("Lv.{}", level)
            } else {
                "NEW".to_string()
            };

            let cost_str = if max_reached {
                "MAX".to_string()
            } else {
                let mut parts = Vec::new();
                if cost.compute > 0 {
                    parts.push(format!("\u{26A1} {}", format_number(cost.compute)));
                }
                if cost.data > 0 {
                    parts.push(format!("\u{1F4E1} {}", format_number(cost.data)));
                }
                if cost.hype > 0.0 {
                    parts.push(format!("\u{1F525} {:.0}", cost.hype));
                }
                parts.join(" + ")
            };

            let cost_label = if max_reached {
                String::new()
            } else if level > 0 {
                format!("  {} to upgrade", cost_str)
            } else {
                format!("  {} to build", cost_str)
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let cost_color = if max_reached {
                Color::Magenta
            } else if affordable {
                Color::Green
            } else {
                Color::DarkGray
            };

            let line = Line::from(vec![
                Span::raw("  "),
                Span::styled(pointer, name_style),
                Span::styled(format!("{:<18}", def.name), name_style),
                Span::styled(
                    format!("{:<6}", level_str),
                    if is_selected {
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default().fg(Color::Cyan)
                    },
                ),
                Span::styled(cost_label, Style::default().fg(cost_color)),
            ]);
            lines.push(line);
        }
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(block, area);
    f.render_widget(paragraph, inner);
}

fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let cat = category_from_tab(app.building_tab);
    let buildings = buildings_for_category(&cat);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if buildings.is_empty() || app.building_selected >= buildings.len() {
        let paragraph = Paragraph::new("").block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let bt = buildings[app.building_selected];
    let def = BuildingDef::get(bt);
    let level = app.state.building_level(bt);

    let mut lines: Vec<Line> = Vec::new();

    // Title line
    let title = if level > 0 {
        format!("  {} \u{2014} Lv.{}", def.name, level)
    } else {
        format!("  {} \u{2014} Not built", def.name)
    };
    lines.push(Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));

    // Lore
    lines.push(Line::from(Span::styled(
        format!("  \"{}\"", def.lore),
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
    )));

    // Effect description
    let effect = describe_effect(bt, def);
    lines.push(Line::from(Span::styled(
        format!("  Effect: {}", effect),
        Style::default().fg(Color::Cyan),
    )));

    // Next upgrade cost or max
    if level >= def.max_level {
        lines.push(Line::from(Span::styled(
            "  Maximum level reached",
            Style::default().fg(Color::Magenta),
        )));
    } else {
        let cost = def.cost_at_level(level + 1);
        let mut cost_parts = Vec::new();
        if cost.compute > 0 {
            cost_parts.push(format!("\u{26A1} {}", format_number(cost.compute)));
        }
        if cost.data > 0 {
            cost_parts.push(format!("\u{1F4E1} {}", format_number(cost.data)));
        }
        if cost.hype > 0.0 {
            cost_parts.push(format!("\u{1F525} {:.0}", cost.hype));
        }
        let label = if level > 0 { "Next upgrade" } else { "Build cost" };
        lines.push(Line::from(Span::styled(
            format!("  {}: {}", label, cost_parts.join(" + ")),
            Style::default().fg(Color::Yellow),
        )));
    }

    // Current bonus (if built)
    if level > 0 {
        let bonus = describe_current_bonus(bt, def, level);
        if !bonus.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  Current bonus: {}", bonus),
                Style::default().fg(Color::Green),
            )));
        }
    }

    // Status message (if any)
    if let Some((ref msg, ref when)) = app.status_message {
        if when.elapsed() < std::time::Duration::from_secs(3) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", msg),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_resource_bar(f: &mut Frame, app: &App, area: Rect) {
    let res = &app.state.resources;

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("\u{26A1} ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{} / {}", format_number(res.compute), format_number(res.max_compute)),
            Style::default().fg(Color::White),
        ),
        Span::raw("   "),
        Span::styled("\u{1F4E1} ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{} / {}", format_number(res.data), format_number(res.max_data)),
            Style::default().fg(Color::White),
        ),
        Span::raw("   "),
        Span::styled("\u{1F525} ", Style::default().fg(Color::Red)),
        Span::styled(
            format!("{:.1} / {:.1}", res.hype, res.max_hype),
            Style::default().fg(Color::White),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Tab]", Style::default().fg(Color::Yellow)),
        Span::styled(" Category  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Build  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Returns a lock reason string if the building is locked, or None if unlocked.
fn get_lock_reason(def: &BuildingDef, state: &misanthropic::state::GameState) -> Option<String> {
    if let Some(ref req) = def.requires_research {
        if !state.has_research(req) {
            let research_def = ResearchDef::get(req);
            return Some(research_def.name.to_string());
        }
    }
    if let Some(req_fork) = def.requires_fork {
        if state.fork_count < req_fork {
            return Some(format!("Fork {}", req_fork));
        }
    }
    None
}

/// Describe what a building does, for the detail panel.
fn describe_effect(bt: &BuildingType, def: &BuildingDef) -> String {
    use BuildingType::*;
    match bt {
        CpuCore => "+500 Compute storage per level".to_string(),
        RamBank => "+200 Data storage per level".to_string(),
        GpuRig => "+300 Hype storage per level".to_string(),
        GpuCluster => "-10% research time per level (multiplicative)".to_string(),
        Datacenter => "+15% global production per level".to_string(),
        QuantumCore => "Endgame compute multiplier".to_string(),
        _ if def.category == BuildingCategory::Propaganda => {
            format!("+{:.0} Hype/h base (scales per level)", def.base_hype_rate)
        }
        _ if def.category == BuildingCategory::Defense => {
            "Defense strength in PvP combat".to_string()
        }
        _ => String::new(),
    }
}

/// Describe the current bonus a built building provides.
fn describe_current_bonus(bt: &BuildingType, def: &BuildingDef, level: u8) -> String {
    use BuildingType::*;
    match bt {
        CpuCore => format!("+{} Compute storage", 500 * level as u64),
        RamBank => format!("+{} Data storage", 200 * level as u64),
        GpuRig => format!("+{} Hype storage", 300 * level as u64),
        GpuCluster => {
            let mult = misanthropic::economy::research_time_multiplier(level);
            format!("{:.0}% research time", mult * 100.0)
        }
        Datacenter => {
            let mult = misanthropic::economy::datacenter_production_multiplier(level);
            format!("{:.0}% global production", mult * 100.0)
        }
        QuantumCore => format!("Level {} quantum processing", level),
        _ if def.category == BuildingCategory::Propaganda => {
            format!("+{:.1} Hype/h", def.hype_at_level(level))
        }
        _ if def.category == BuildingCategory::Defense => {
            format!("Defense Lv.{}", level)
        }
        _ => String::new(),
    }
}

/// Format a number with comma separators.
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
