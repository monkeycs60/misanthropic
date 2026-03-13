use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::App;

pub fn render_combat_menu(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" COMBAT ")
        .title_alignment(Alignment::Center);

    let title_text = Line::from(Span::styled(
        "Choose your battlefield",
        Style::default().fg(Color::DarkGray),
    ));
    let title_para = Paragraph::new(title_text)
        .block(title_block)
        .alignment(Alignment::Center);
    f.render_widget(title_para, chunks[0]);

    // Menu items
    let content_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = content_block.inner(chunks[1]);
    f.render_widget(content_block, chunks[1]);

    let items = [
        ("PvE \u{2014} Tower Climb", "Fight through layers of human resistance sector by sector"),
        ("PvP \u{2014} Hype Battles", "Raid other AIs for resources (requires online backend)"),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (i, (label, desc)) in items.iter().enumerate() {
        let is_selected = app.combat_menu_selected == i;
        let pointer = if is_selected { "\u{25B8} " } else { "  " };

        let name_style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(pointer, name_style),
            Span::styled(label.to_string(), name_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("        "),
            Span::styled(
                desc.to_string(),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]));
        lines.push(Line::from(""));
    }

    // PvP notice
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("    "),
        Span::styled(
            "Note: PvP requires online backend (coming soon)",
            Style::default().fg(Color::DarkGray),
        ),
    ]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);

    // Help line
    let help = Line::from(vec![
        Span::styled(" [\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Select  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Choose  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);
    let help_para = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(help_para, chunks[2]);
}
