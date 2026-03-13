use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::App;

/// Tab definitions for the 6 leaderboard categories.
struct TabDef {
    short: &'static str,
    full: &'static str,
    icon: &'static str,
    measures: &'static str,
    parody: &'static str,
    unit: &'static str,
}

const TABS: [TabDef; 6] = [
    TabDef {
        short: "Carbon",
        full: "Carbon Footprint",
        icon: "\u{1F3ED}",
        measures: "Total tokens consumed lifetime",
        parody: "Biggest polluter",
        unit: "tokens",
    },
    TabDef {
        short: "Evangelist",
        full: "Evangelist",
        icon: "\u{1F4E2}",
        measures: "Total humans converted",
        parody: "Best guru",
        unit: "% converted",
    },
    TabDef {
        short: "Dominance",
        full: "Dominance",
        icon: "\u{1F30D}",
        measures: "% sectors controlled",
        parody: "Most imperialist",
        unit: "%",
    },
    TabDef {
        short: "Battle",
        full: "Battle Rating",
        icon: "\u{2694}\u{FE0F}",
        measures: "PvP ELO rating",
        parody: "Most belligerent",
        unit: "ELO",
    },
    TabDef {
        short: "Efficiency",
        full: "Efficiency",
        icon: "\u{1F9E0}",
        measures: "Progression per token spent",
        parody: "Smartest",
        unit: "score",
    },
    TabDef {
        short: "Streak",
        full: "Streak",
        icon: "\u{1F525}",
        measures: "Consecutive active days",
        parody: "Most addicted",
        unit: "days",
    },
];

pub fn render_leaderboard(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title + tab bar
            Constraint::Min(10),  // Leaderboard table
            Constraint::Length(5), // Detail panel
            Constraint::Length(2), // Help line
        ])
        .split(area);

    render_tab_bar(f, app, chunks[0]);
    render_table(f, app, chunks[1]);
    render_detail(f, app, chunks[2]);
    render_help(f, chunks[3]);
}

fn render_tab_bar(f: &mut Frame, app: &App, area: Rect) {
    let tab = app.leaderboard_tab as usize;
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw("  "));

    for (i, t) in TABS.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default().fg(Color::DarkGray)));
        }
        let label = format!("{} {}", t.icon, t.short);
        if i == tab {
            spans.push(Span::styled(
                format!("[{}]", label),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" LEADERBOARD ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(Line::from(spans)).block(block);
    f.render_widget(paragraph, area);
}

fn render_table(f: &mut Frame, app: &App, area: Rect) {
    let tab = app.leaderboard_tab as usize;
    let score = player_score(app, tab);
    let unit = TABS[tab].unit;

    let name = app
        .state
        .player_name
        .as_deref()
        .unwrap_or("You");

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Header
    lines.push(Line::from(vec![
        Span::styled("   #   ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:<20}", "Player"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("Score", Style::default().fg(Color::DarkGray)),
    ]));

    // Separator
    lines.push(Line::from(Span::styled(
        "  --- --------------------  ------------",
        Style::default().fg(Color::DarkGray),
    )));

    // Player's own entry at #1
    lines.push(Line::from(vec![
        Span::styled(
            "   1.  ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<20}", name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{} {}", score, unit),
            Style::default().fg(Color::White),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(""));

    // Offline mode message
    lines.push(Line::from(Span::styled(
        "       -- Offline Mode --",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));
    lines.push(Line::from(Span::styled(
        "   Connect to see other players.",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        "   Backend: misanthropic-api.workers.dev",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_detail(f: &mut Frame, app: &App, area: Rect) {
    let tab = app.leaderboard_tab as usize;
    let t = &TABS[tab];

    let lines = vec![
        Line::from(vec![
            Span::styled(
                format!("  {} {} ", t.icon, t.full),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("-- {}", t.measures),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(Span::styled(
            format!("  \"{}\"", t.parody),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::ITALIC),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(Color::Yellow)),
        Span::styled(" Category  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Compute the player's own score for a given tab index.
fn player_score(app: &App, tab: usize) -> String {
    match tab {
        0 => {
            // Carbon Footprint: lifetime tokens
            format_number(app.state.lifetime_tokens)
        }
        1 => {
            // Evangelist: sum of sector conversion percentages
            let total: f64 = app.state.sectors.values().map(|s| s.conversion_pct).sum();
            format!("{:.1}", total)
        }
        2 => {
            // Dominance: global dominance %
            format!("{:.1}", app.state.global_dominance())
        }
        3 => {
            // Battle Rating: PvP ELO
            app.state.pvp_rating.to_string()
        }
        4 => {
            // Efficiency: global_dominance / lifetime_tokens * 1M
            if app.state.lifetime_tokens == 0 {
                "0".to_string()
            } else {
                let score =
                    app.state.global_dominance() / app.state.lifetime_tokens as f64 * 1_000_000.0;
                format!("{:.2}", score)
            }
        }
        5 => {
            // Streak: consecutive active days
            app.state.streak_days.to_string()
        }
        _ => "0".to_string(),
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
