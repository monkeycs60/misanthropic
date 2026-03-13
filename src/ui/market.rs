use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::App;

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
        .title(" [Esc] Dashboard | BLACK MARKET ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(Line::from(Span::styled(
        " Convert $ into resources",
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

    let trades: Vec<(&str, &str, String, String, Color)> = vec![
        (
            "\u{1F4E1}",
            "Buy Data",
            format!("${}/unit", format_k(data_price)),
            format!("{} bought", app.state.data_bought),
            Color::Cyan,
        ),
        (
            "\u{1F525}",
            "Buy Hype",
            format!("${}/unit", format_k(hype_price)),
            format!("{} bought", app.state.hype_bought),
            Color::Red,
        ),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Amount options per trade
    let amounts = [1u32, 5, 10, 25];

    for (trade_idx, (icon, name, price, history, color)) in trades.iter().enumerate() {
        let is_selected = trade_idx == app.market_selected;
        let pointer = if is_selected { "\u{25B8} " } else { "  " };

        // Trade header
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
            Span::styled(format!("  {}", price), Style::default().fg(*color)),
            Span::styled(format!("  ({})", history), Style::default().fg(Color::DarkGray)),
        ]));

        // Amount buttons row
        if is_selected {
            let mut btn_spans: Vec<Span> = vec![Span::raw("      ")];

            for (btn_idx, &amt) in amounts.iter().enumerate() {
                let is_btn_selected = btn_idx == app.market_amount_idx;
                let total_cost = if trade_idx == 0 {
                    misanthropic::economy::trade_cost(1_000, app.state.data_bought, amt)
                } else {
                    misanthropic::economy::trade_cost(5_000, app.state.hype_bought, amt)
                };
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

    // Price explanation
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Prices rise 3% per unit bought (supply & demand)",
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
