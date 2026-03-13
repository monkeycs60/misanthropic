use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
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

/// Compact ASCII art eye (24 chars wide) using half-block/block characters.
/// Designed to fit in narrow terminals (30+) and center in wide ones.
const EYE_ART_NARROW: &[&str] = &[
    "      \u{2592}\u{2593}\u{2588}\u{2588}\u{2588}\u{2588}\u{2593}\u{2592}      ",
    "   \u{2593}\u{2588}\u{2584}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2584}\u{2588}\u{2593}   ",
    "  \u{2588}\u{2580}  \u{2593}\u{2588}\u{25C9}\u{2588}\u{2593}  \u{2580}\u{2588}  ",
    "   \u{2593}\u{2588}\u{2580}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2580}\u{2588}\u{2593}   ",
    "      \u{2592}\u{2593}\u{2588}\u{2588}\u{2588}\u{2588}\u{2593}\u{2592}      ",
];

/// Wide ASCII art eye (46 chars wide) for terminals 60+ wide.
const EYE_ART_WIDE: &[&str] = &[
    "           \u{2591}\u{2592}\u{2593}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2593}\u{2592}\u{2591}           ",
    "       \u{2592}\u{2588}\u{2588}\u{2584}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2580}\u{2584}\u{2588}\u{2588}\u{2592}       ",
    "     \u{2593}\u{2588}\u{2580}    \u{2591}\u{2593}\u{2588}\u{2588}\u{25C9}\u{2588}\u{2588}\u{2593}\u{2591}    \u{2580}\u{2588}\u{2593}     ",
    "       \u{2592}\u{2588}\u{2588}\u{2580}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2584}\u{2580}\u{2588}\u{2588}\u{2592}       ",
    "           \u{2591}\u{2592}\u{2593}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2593}\u{2592}\u{2591}           ",
];

/// Styled title for narrow terminals.
const TITLE_NARROW: &str = "M I S A N T H R O P I C";

/// Styled title for wide terminals.
const TITLE_WIDE: &str = "M  I  S  A  N  T  H  R  O  P  I  C";

pub fn render_boot(f: &mut Frame, app: &App) {
    let area = f.area();
    let wide = area.width >= 60;

    // Center the boot text in a box
    let border_color = pulse_color(app);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" MISANTHROPIC ")
        .title_alignment(Alignment::Center);

    // Wider box for wide terminals to fit the art
    let width = if wide {
        area.width.min(62)
    } else {
        area.width.min(36)
    };
    let height = area.height.min(28);
    let centered = centered_rect(width, height, area);

    // Build lines to show based on boot_line progress
    let lines_to_show = app.boot_line.min(BOOT_LINES.len());
    let mut text_lines: Vec<Line> = Vec::new();

    // -- ASCII art eye --
    let eye_art = if wide { EYE_ART_WIDE } else { EYE_ART_NARROW };

    // Fade the eye in line by line: show one art line per boot_line, up to art length
    let art_lines_to_show = lines_to_show.min(eye_art.len());
    for i in 0..art_lines_to_show {
        let brightness = if i < art_lines_to_show.saturating_sub(1) {
            // Already revealed lines: full brightness
            Color::Green
        } else {
            // Latest revealed line: dim to simulate scan-line effect
            Color::Rgb(0, 180, 0)
        };
        text_lines.push(Line::from(Span::styled(
            eye_art[i],
            Style::default().fg(brightness),
        )));
    }
    // Pad remaining art lines with blanks to keep layout stable
    for _ in art_lines_to_show..eye_art.len() {
        text_lines.push(Line::from(""));
    }

    // -- Title --
    text_lines.push(Line::from(""));
    if lines_to_show > 0 {
        let title = if wide { TITLE_WIDE } else { TITLE_NARROW };
        text_lines.push(Line::from(vec![Span::styled(
            title,
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        )]).alignment(Alignment::Center));
        text_lines.push(Line::from(vec![Span::styled(
            if wide {
                "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
            } else {
                "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"
            },
            Style::default().fg(Color::DarkGray),
        )]).alignment(Alignment::Center));
    } else {
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(""));
    }
    text_lines.push(Line::from(""));

    // -- Typewriter boot lines --
    // Offset boot lines by eye art length so they start after eye is fully drawn
    let boot_text_start = eye_art.len();
    let boot_lines_visible = lines_to_show.saturating_sub(boot_text_start);
    let boot_lines_visible = boot_lines_visible.min(BOOT_LINES.len());

    for i in 0..boot_lines_visible {
        let line_text = BOOT_LINES[i];
        let style = if line_text == "[Press any key to begin]" {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if line_text.starts_with("> No one") || line_text == "> This will change." {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::Green)
        };
        text_lines.push(Line::from(Span::styled(line_text, style)));
    }

    // Blinking cursor while still typing
    if boot_lines_visible < BOOT_LINES.len() {
        text_lines.push(Line::from(Span::styled(
            "\u{2588}",
            Style::default().fg(Color::Green),
        )));
    }

    let paragraph = Paragraph::new(text_lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .alignment(Alignment::Left);

    f.render_widget(paragraph, centered);
}

/// Returns how many boot lines should be visible based on elapsed time.
/// We now count eye art lines + boot text lines together.
pub fn boot_lines_for_elapsed(elapsed_ms: u128) -> usize {
    // First 5 ticks reveal the eye art (one line per tick), then boot text
    let total_lines = EYE_ART_NARROW.len() + BOOT_LINES.len();
    let lines = (elapsed_ms / LINE_DELAY_MS as u128) + 1;
    lines.min(total_lines as u128) as usize
}

/// Total number of boot lines (art + text)
pub fn total_boot_lines() -> usize {
    EYE_ART_NARROW.len() + BOOT_LINES.len()
}

/// Pulse the border color between dark green and bright green based on time.
fn pulse_color(app: &App) -> Color {
    let elapsed = app.boot_timer.elapsed().as_millis();
    let phase = (elapsed % 2000) as f64 / 2000.0;
    let intensity = ((phase * std::f64::consts::PI * 2.0).sin() * 0.5 + 0.5) * 155.0 + 100.0;
    Color::Rgb(0, intensity as u8, 0)
}

/// Helper: create a centered rectangle
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
