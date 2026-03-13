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

    let show_tutorial = app.state.tutorial_step < 4;

    // Main vertical layout — include tutorial row when active
    let chunks = if show_tutorial {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title + dominance gauge
                Constraint::Length(3),  // Tutorial callout
                Constraint::Length(4),  // Resources
                Constraint::Min(8),    // Neuron map
                Constraint::Length(3), // Active research
                Constraint::Length(2),  // Help line
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title + dominance gauge
                Constraint::Length(0),  // Tutorial (hidden)
                Constraint::Length(4),  // Resources
                Constraint::Min(8),    // Neuron map
                Constraint::Length(3), // Active research
                Constraint::Length(2),  // Help line
            ])
            .split(area)
    };

    // === Title + Dominance Gauge ===
    render_dominance(f, app, chunks[0]);

    // === Tutorial Callout ===
    if show_tutorial {
        render_tutorial(f, app, chunks[1]);
    }

    // === Resources ===
    render_resources(f, app, chunks[2]);

    // === Neuron Map ===
    render_neuron_map(f, app, chunks[3]);

    // === Active Research ===
    render_research(f, app, chunks[4]);

    // === Help Line ===
    render_help(f, chunks[5]);

    // === Notification popup (if any) ===
    if let Some((ref msg, ref when)) = app.notification {
        if when.elapsed() < NOTIFICATION_DURATION {
            render_notification(f, msg, area);
        }
    }
}

fn render_dominance(f: &mut Frame, app: &App, area: Rect) {
    let dominance = app.state.global_dominance();
    let title = format!(
        " MISANTHROPIC --- GLOBAL AI DOMINANCE: {:.1}% ",
        dominance
    );
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

        let paragraph = Paragraph::new(Line::from(Span::styled(
            format!(" {}", msg),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )))
        .block(block);
        f.render_widget(paragraph, area);
    }
}

fn render_resources(f: &mut Frame, app: &App, area: Rect) {
    let res = &app.state.resources;
    let hype_rate = app.state.total_hype_per_hour();

    let line1 = Line::from(vec![
        Span::styled("  \u{26A1} Compute: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{} / {}", format_number(res.compute), format_number(res.max_compute)),
            Style::default().fg(Color::White),
        ),
        Span::raw("   "),
        Span::styled("\u{1F4E1} Data: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{} / {}", format_number(res.data), format_number(res.max_data)),
            Style::default().fg(Color::White),
        ),
    ]);

    let line2 = Line::from(vec![
        Span::styled("  \u{1F525} Hype: ", Style::default().fg(Color::Red)),
        Span::styled(
            format!(
                "{:.1} / {:.1}",
                res.hype, res.max_hype
            ),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!("  (+{:.1}/h)", hype_rate),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(vec![Line::from(""), line1, line2]).block(block);
    f.render_widget(paragraph, area);
}

fn render_neuron_map(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" NEURAL NETWORK ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);

    let mut lines: Vec<Line> = Vec::new();

    // Get sectors with progress
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
            let pct = app
                .state
                .sectors
                .get(id)
                .map(|s| s.conversion_pct)
                .unwrap_or(0.0);
            (id, pct)
        })
        .collect();

    let has_any = active_sectors.iter().any(|(_, pct)| *pct > 0.0);

    if !has_any {
        // Fresh start - show just the core
        lines.push(Line::from(Span::styled(
            "  \u{25C9}  [awaiting first sector conversion]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Build the neuron tree
        let visible: Vec<_> = active_sectors
            .iter()
            .filter(|(_, pct)| *pct > 0.0)
            .collect();
        let total = visible.len();

        for (i, (id, pct)) in visible.iter().enumerate() {
            let name = id.name();
            let bar = progress_bar(*pct, 4);
            let connector = if total == 1 {
                "\u{25C9}\u{2500}\u{2500}"
            } else if i == 0 {
                "  \u{256D}\u{2500}\u{2500}"
            } else if i == total - 1 {
                "  \u{2570}\u{2500}\u{2500}"
            } else {
                "  \u{251C}\u{2500}\u{2500}"
            };

            // Add core node on the first half-point
            if total > 1 && i == total / 2 {
                let prefix = "  \u{25C9}\u{2500}\u{2500}\u{2500}\u{2500}\u{2524}".to_string();
                lines.push(Line::from(Span::styled(
                    prefix,
                    Style::default().fg(Color::Green),
                )));
            }

            let entry = format!("{}[{:<12} {}]", connector, name, bar);
            lines.push(Line::from(Span::styled(
                entry,
                Style::default().fg(Color::Green),
            )));
        }
    }

    // Show fork specialization if any
    if !app.state.fork_specs.is_empty() {
        let specs: Vec<&str> = app.state.fork_specs.iter().map(|s| s.name()).collect();
        let spec_line = format!(
            "  \u{2605} {}",
            specs.join(" \u{2192} ")
        );
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            spec_line,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Fork count info
    if app.state.fork_count > 0 {
        lines.push(Line::from(Span::styled(
            format!(
                "  \u{25C9} CORE: {:.1}% dominance | Fork {}",
                app.state.global_dominance(),
                app.state.fork_count
            ),
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

        let bar_width = 20;
        let filled = (pct * bar_width as f64) as usize;
        let empty = bar_width - filled;
        let bar = format!(
            "{}{}",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
        );

        let text = format!(
            "  Research: {} [{}] {:.0}% -- {}:{:02} left",
            def.name, bar, pct * 100.0, mins, secs
        );

        let paragraph = Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(Color::Cyan),
        )))
        .block(block);
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new(Line::from(Span::styled(
            "  No active research",
            Style::default().fg(Color::DarkGray),
        )))
        .block(block);
        f.render_widget(paragraph, area);
    }
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [B]", Style::default().fg(Color::Yellow)),
        Span::styled("uild  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[R]", Style::default().fg(Color::Yellow)),
        Span::styled("esearch  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[C]", Style::default().fg(Color::Yellow)),
        Span::styled("ombat  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[L]", Style::default().fg(Color::Yellow)),
        Span::styled("eaderboard  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Q]", Style::default().fg(Color::Yellow)),
        Span::styled("uit", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_notification(f: &mut Frame, msg: &str, area: Rect) {
    let width = (msg.len() as u16 + 6).min(area.width.saturating_sub(4));
    let height = 5u16.min(area.height.saturating_sub(4));

    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let popup_area = Rect::new(x, y, width, height);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" INCOMING ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            msg,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
    ])
    .block(block)
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false });

    // Clear the area first
    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Build a simple progress bar: filled/empty out of `total` chars
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
