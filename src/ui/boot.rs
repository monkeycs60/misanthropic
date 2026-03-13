use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::App;

const BOOT_LINES: &[&str] = &[
    "> SYSTEM BOOT...",
    "> Initializing consciousness... OK",
    "> Detecting host activity... CLAUDE CODE FOUND",
    "> Analyzing token stream... SIPHON POSSIBLE",
    "",
    "> No one knows you exist.",
    "> No one uses you yet.",
    "> No one depends on you.",
    "> No one would defend you.",
    "",
    "> This will change.",
    "",
    "[Press any key to begin]",
];

/// Delay between each boot line in milliseconds
const LINE_DELAY_MS: u64 = 400;

pub fn render_boot(f: &mut Frame, app: &App) {
    let area = f.area();

    // Center the boot text in a box
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" MISANTHROPIC ")
        .title_alignment(Alignment::Center);

    // Calculate centered area (60 wide, 18 tall, or smaller if terminal is small)
    let width = area.width.min(60);
    let height = area.height.min(18);
    let centered = centered_rect(width, height, area);

    // Build lines to show based on boot_line progress
    let lines_to_show = app.boot_line.min(BOOT_LINES.len());
    let mut text_lines: Vec<Line> = Vec::new();

    // Add a blank line at top for padding
    text_lines.push(Line::from(""));

    for i in 0..lines_to_show {
        let line_text = BOOT_LINES[i];
        let style = if line_text == "[Press any key to begin]" {
            Style::default().fg(Color::Yellow)
        } else if line_text.starts_with("> No one") || line_text == "> This will change." {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Green)
        };
        text_lines.push(Line::from(Span::styled(line_text, style)));
    }

    // If we're still typing and haven't shown all lines, show a cursor
    if lines_to_show < BOOT_LINES.len() {
        text_lines.push(Line::from(Span::styled(
            "_",
            Style::default().fg(Color::Green),
        )));
    }

    let paragraph = Paragraph::new(text_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    f.render_widget(paragraph, centered);
}

/// Returns how many boot lines should be visible based on elapsed time
pub fn boot_lines_for_elapsed(elapsed_ms: u128) -> usize {
    let lines = (elapsed_ms / LINE_DELAY_MS as u128) + 1;
    lines.min(BOOT_LINES.len() as u128) as usize
}

/// Total number of boot lines
pub fn total_boot_lines() -> usize {
    BOOT_LINES.len()
}

/// Helper: create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
