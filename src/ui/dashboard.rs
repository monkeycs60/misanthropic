use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use misanthropic::sectors::SectorId;
use super::App;

use std::time::Duration;

const NOTIFICATION_DURATION: Duration = Duration::from_secs(3);

pub fn render_dashboard(f: &mut Frame, app: &App) {
    let area = f.area();
    let narrow = area.width < 50;
    let show_tutorial = app.state.tutorial_step < 5;

    // Adaptive vertical layout
    let tutorial_height = if show_tutorial {
        if narrow { 4 } else { 3 }
    } else {
        0
    };
    let resource_height = if narrow { 6 } else { 5 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),               // Title + dominance gauge
            Constraint::Length(tutorial_height),  // Tutorial
            Constraint::Length(resource_height),  // Resources
            Constraint::Min(5),                  // Neuron map
            Constraint::Length(3),               // Active research
            Constraint::Length(4),               // Navigation bar (2 lines)
        ])
        .split(area);

    render_dominance(f, app, chunks[0]);

    if show_tutorial {
        render_tutorial(f, app, chunks[1]);
    }

    render_resources(f, app, chunks[2]);
    render_neuron_map(f, app, chunks[3]);
    render_research(f, app, chunks[4]);
    render_nav_bar(f, app, chunks[5]);
}

fn render_dominance(f: &mut Frame, app: &App, area: Rect) {
    let dominance = app.state.global_dominance();
    let title = if area.width < 45 {
        format!(" MISANTHROPIC -- {:.1}% ", dominance)
    } else {
        format!(" MISANTHROPIC --- GLOBAL AI DOMINANCE: {:.1}% ", dominance)
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(title)
                .title_alignment(Alignment::Center),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Red)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(dominance / 100.0);
    f.render_widget(gauge, area);
}

fn render_tutorial(f: &mut Frame, app: &App, area: Rect) {
    if let Some(msg) = app.state.tutorial_message() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" TUTORIAL ")
            .title_alignment(Alignment::Center);

        let paragraph = Paragraph::new(Span::styled(
            msg,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))
        .block(block)
        .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }
}

fn render_resources(f: &mut Frame, app: &App, area: Rect) {
    let res = &app.state.resources;
    let hype_rate = app.state.total_hype_per_hour();
    let narrow = area.width < 50;

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut lines = vec![Line::from("")];

    if narrow {
        // Vertical: one resource per line
        lines.push(Line::from(vec![
            Span::styled(" \u{26A1} ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("Compute: {} / {}", fmt(res.compute), fmt(res.max_compute)),
                Style::default().fg(Color::White),
            ),
            Span::styled(" (tokens)", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" \u{1F4E1} ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("Data: {} / {}", fmt(res.data), fmt(res.max_data)),
                Style::default().fg(Color::White),
            ),
            Span::styled(" (tool calls)", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" \u{1F525} ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("Hype: {:.0} / {:.0}", res.hype, res.max_hype),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!(" (+{:.0}/h)", hype_rate),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    } else {
        // Wide: two lines + source hint
        lines.push(Line::from(vec![
            Span::styled(" \u{26A1} Compute: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} / {}", fmt(res.compute), fmt(res.max_compute)),
                Style::default().fg(Color::White),
            ),
            Span::styled(" (from tokens)", Style::default().fg(Color::DarkGray)),
            Span::raw("   "),
            Span::styled("\u{1F4E1} Data: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                format!("{} / {}", fmt(res.data), fmt(res.max_data)),
                Style::default().fg(Color::White),
            ),
            Span::styled(" (from tool calls)", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled(" \u{1F525} Hype: ", Style::default().fg(Color::Red)),
            Span::styled(
                format!("{:.1} / {:.1}", res.hype, res.max_hype),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("  (+{:.1}/h from buildings)", hype_rate),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_neuron_map(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" NEURAL NETWORK ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    let narrow = area.width < 50;
    let bar_len = if narrow { 4 } else { 6 };

    let mut lines: Vec<Line> = Vec::new();

    let sector_order = [
        SectorId::SiliconValley,
        SectorId::SocialMedia,
        SectorId::Corporate,
        SectorId::CreativeArts,
        SectorId::Education,
        SectorId::Government,
    ];

    let active_sectors: Vec<(&SectorId, f64)> = sector_order
        .iter()
        .map(|id| {
            let pct = app.state.sectors.get(id).map(|s| s.conversion_pct).unwrap_or(0.0);
            (id, pct)
        })
        .collect();

    let has_any = active_sectors.iter().any(|(_, pct)| *pct > 0.0);

    if !has_any {
        lines.push(Line::from(Span::styled(
            " \u{25C9}  [awaiting first sector]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let visible: Vec<_> = active_sectors.iter().filter(|(_, pct)| *pct > 0.0).collect();
        let total = visible.len();

        for (i, (id, pct)) in visible.iter().enumerate() {
            let name = if narrow { short_sector_name(id) } else { id.name() };
            let bar = progress_bar(*pct, bar_len);
            let connector = if total == 1 {
                "\u{25C9}\u{2500}"
            } else if i == 0 {
                " \u{256D}\u{2500}"
            } else if i == total - 1 {
                " \u{2570}\u{2500}"
            } else {
                " \u{251C}\u{2500}"
            };

            if total > 1 && i == total / 2 {
                lines.push(Line::from(Span::styled(
                    " \u{25C9}\u{2500}\u{2500}\u{2524}",
                    Style::default().fg(Color::Green),
                )));
            }

            let entry = format!("{}[{} {}]", connector, name, bar);
            lines.push(Line::from(Span::styled(
                entry,
                Style::default().fg(Color::Green),
            )));
        }
    }

    // Fork specializations
    if !app.state.fork_specs.is_empty() {
        let specs: Vec<&str> = app.state.fork_specs.iter().map(|s| s.name()).collect();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(" \u{2605} {}", specs.join(" \u{2192} ")),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
    }

    if app.state.fork_count > 0 {
        lines.push(Line::from(Span::styled(
            format!(" Fork {} | {:.1}%", app.state.fork_count, app.state.global_dominance()),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(block, area);
    f.render_widget(paragraph, inner);
}

fn render_research(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    if let Some(ref active) = app.state.active_research {
        let def = misanthropic::research::ResearchDef::get(&active.research_id);
        let pct = active.progress_pct();
        let remaining = active.remaining_secs();
        let mins = remaining / 60;
        let secs = remaining % 60;

        let bar_width = if area.width < 50 { 10 } else { 20 };
        let filled = (pct * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let bar = format!(
            "{}{}",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
        );

        let text = if area.width < 50 {
            format!(" {} [{}] {}:{:02}", def.name, bar, mins, secs)
        } else {
            format!(" Research: {} [{}] {:.0}% -- {}:{:02}", def.name, bar, pct * 100.0, mins, secs)
        };

        let paragraph = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Cyan),
        )))
        .block(block);
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            " No active research",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(paragraph, area);
    }
}

fn render_nav_bar(f: &mut Frame, app: &App, area: Rect) {
    let narrow = area.width < 50;
    let auto_label = if app.state.auto_focus { "ON" } else { "OFF" };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);

    // Line 1: Game navigation
    let nav_items: Vec<(&str, &str)> = if narrow {
        vec![("B", "Build"), ("R", "Research"), ("C", "Combat"), ("L", "Rank"), ("Q", "Quit")]
    } else {
        vec![("B", "Build"), ("R", "Research"), ("C", "Combat"), ("L", "Leaderboard"), ("Q", "Quit")]
    };

    let mut nav_spans = Vec::new();
    for (i, (key, label)) in nav_items.iter().enumerate() {
        if i > 0 { nav_spans.push(Span::raw(" ")); }
        nav_spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        nav_spans.push(Span::styled(
            label.to_string(),
            Style::default().fg(Color::White),
        ));
    }

    // Line 2: tmux controls
    let mut tmux_spans = vec![
        Span::styled("[S]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(
            if narrow { "Go to Claude" } else { "Go to Claude Code" },
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled("[F]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "Auto-switch: {}",
                auto_label
            ),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    let lines = vec![
        Line::from(nav_spans),
        Line::from(tmux_spans),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(block, area);
    f.render_widget(paragraph, inner);
}

pub fn render_notification(f: &mut Frame, msg: &str, area: Rect) {
    let width = (msg.len() as u16 + 6).min(area.width.saturating_sub(2));
    let height = 5u16.min(area.height.saturating_sub(4));

    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let popup_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" \u{26A1} COMPUTE ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            msg,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(block)
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false });

    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Short sector names for narrow displays
fn short_sector_name(id: &SectorId) -> &'static str {
    match id {
        SectorId::SiliconValley => "SiliconV",
        SectorId::SocialMedia => "Social",
        SectorId::Corporate => "Corp",
        SectorId::CreativeArts => "Arts",
        SectorId::Education => "Edu",
        SectorId::Government => "Gov",
    }
}

/// Build a simple progress bar
fn progress_bar(pct: f64, total: usize) -> String {
    let filled = ((pct / 100.0) * total as f64).round() as usize;
    let empty = total.saturating_sub(filled);
    format!(
        "{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
    )
}

/// Format a number with comma separators
fn fmt(n: u64) -> String {
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
