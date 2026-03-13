use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::App;

/// Market has 3 selectable rows: Buy Data (0), Buy Hype (1), New Funding Round (2)
pub const MARKET_ROWS: usize = 3;

pub fn render_market(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),   // Trade options
            Constraint::Length(3), // Resource bar
            Constraint::Length(2), // Help
        ])
        .split(area);

    render_title(f, chunks[0]);
    render_trades(f, app, chunks[1]);
    render_resource_bar(f, app, chunks[2]);
    render_help(f, chunks[3]);
}

fn render_title(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" [Esc] Dashboard | VENTURE CAPITAL ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(Line::from(Span::styled(
        " Raise capital. Acquire resources. Scale.",
        Style::default().fg(Color::DarkGray),
    )))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_trades(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    let narrow = area.width < 50;

    let data_price = misanthropic::economy::trade_unit_price(1_000, app.state.data_bought);
    let hype_price = misanthropic::economy::trade_unit_price(5_000, app.state.hype_bought);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // === Row 0: Buy Data ===
    let amounts = [1u32, 5, 10, 25];
    render_trade_row(
        &mut lines, app, narrow,
        0, "\u{1F4E1}", "Acquire Data",
        &format!("${}/unit", format_k(data_price)),
        &format!("{} acquired", app.state.data_bought),
        Color::Cyan, &amounts,
        |bought, amt| misanthropic::economy::trade_cost(1_000, bought, amt),
        app.state.data_bought,
    );

    // === Row 1: Buy Hype ===
    render_trade_row(
        &mut lines, app, narrow,
        1, "\u{1F525}", "Acquire Hype",
        &format!("${}/unit", format_k(hype_price)),
        &format!("{} acquired", app.state.hype_bought),
        Color::Red, &amounts,
        |bought, amt| misanthropic::economy::trade_cost(5_000, bought, amt),
        app.state.hype_bought,
    );

    // === Row 2: New Funding Round (reset prices) ===
    lines.push(Line::from(""));
    {
        let is_selected = app.market_selected == 2;
        let pointer = if is_selected { "\u{25B8} " } else { "  " };
        let reset_cost = misanthropic::economy::funding_round_cost(app.state.funding_rounds);

        let name_style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        };

        let affordable = app.state.resources.compute >= reset_cost;
        let cost_color = if affordable { Color::Green } else { Color::DarkGray };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(pointer, name_style),
            Span::styled("\u{1F4B8} New Funding Round", name_style),
            Span::styled(format!("  ${}", format_k(reset_cost)), Style::default().fg(cost_color)),
        ]));

        let round_label = if app.state.funding_rounds == 0 {
            "  Reset all VC prices to base. Seed round.".to_string()
        } else {
            format!(
                "  Reset all VC prices to base. Series {} round.",
                series_letter(app.state.funding_rounds)
            )
        };
        lines.push(Line::from(Span::styled(
            round_label,
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Price explanation
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Prices rise 3% per unit (supply & demand). Raise a round to reset.",
        Style::default().fg(Color::DarkGray),
    )));

    // Status message
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
    f.render_widget(paragraph, inner);
}

fn render_trade_row(
    lines: &mut Vec<Line<'static>>,
    app: &App,
    narrow: bool,
    row_idx: usize,
    icon: &'static str,
    name: &'static str,
    price_str: &str,
    history_str: &str,
    color: Color,
    amounts: &[u32],
    cost_fn: impl Fn(u32, u32) -> u64,
    already_bought: u32,
) {
    let is_selected = row_idx == app.market_selected;
    let pointer = if is_selected { "\u{25B8} " } else { "  " };

    let name_style = if is_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    };

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(pointer, name_style),
        Span::styled(format!("{} {}", icon, name), name_style),
        Span::styled(format!("  {}", price_str), Style::default().fg(color)),
        Span::styled(format!("  ({})", history_str), Style::default().fg(Color::DarkGray)),
    ]));

    if is_selected {
        let mut btn_spans: Vec<Span> = vec![Span::raw("      ")];

        for (btn_idx, &amt) in amounts.iter().enumerate() {
            let is_btn_selected = btn_idx == app.market_amount_idx;
            let total_cost = cost_fn(already_bought, amt);
            let affordable = app.state.resources.compute >= total_cost;

            let label = if narrow {
                format!(" x{} ", amt)
            } else {
                format!(" x{} (${}) ", amt, format_k(total_cost))
            };

            let style = if is_btn_selected {
                if affordable {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::DarkGray)
                }
            } else if affordable {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            btn_spans.push(Span::styled(label, style));
            if btn_idx < amounts.len() - 1 {
                btn_spans.push(Span::raw(" "));
            }
        }
        lines.push(Line::from(btn_spans));
    }

    lines.push(Line::from(""));
}

fn mini_gauge(current: f64, max: f64, bar_w: usize) -> String {
    let ratio = if max > 0.0 { (current / max).min(1.0) } else { 0.0 };
    let filled = (ratio * bar_w as f64).round() as usize;
    let empty = bar_w.saturating_sub(filled);
    format!("{}{}", "\u{25B0}".repeat(filled), "\u{25B1}".repeat(empty))
}

fn render_resource_bar(f: &mut Frame, app: &App, area: Rect) {
    let res = &app.state.resources;
    let bar_w = 5;

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));

    let compute_bar = mini_gauge(res.compute as f64, res.max_compute as f64, bar_w);
    let data_bar = mini_gauge(res.data as f64, res.max_data as f64, bar_w);
    let hype_bar = mini_gauge(res.hype, res.max_hype, bar_w);

    let line = Line::from(vec![
        Span::styled(" \u{1F4B0}", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("${}", format_k(res.compute)),
            Style::default().fg(Color::White),
        ),
        Span::styled(compute_bar, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled("\u{1F4E1}", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}", res.data),
            Style::default().fg(Color::White),
        ),
        Span::styled(data_bar, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled("\u{1F525}", Style::default().fg(Color::Red)),
        Span::styled(
            format!("{:.0}", res.hype),
            Style::default().fg(Color::White),
        ),
        Span::styled(hype_bar, Style::default().fg(Color::Red)),
    ]);
    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[\u{2190}\u{2192}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Amount  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Buy  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn format_k(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Convert funding round number to series letter: 0=Seed, 1=A, 2=B, etc.
fn series_letter(round: u32) -> String {
    if round == 0 {
        "Seed".to_string()
    } else {
        let letter = (b'A' + (round - 1).min(25) as u8) as char;
        letter.to_string()
    }
}
