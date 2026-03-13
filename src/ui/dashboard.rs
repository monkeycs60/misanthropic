use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Frame,
};

use misanthropic::buildings::BuildingType;
use misanthropic::sectors::SectorId;
use super::App;

/// Map dominance (0..100) to a color.
fn dominance_color(dominance: f64) -> Color {
    if dominance >= 75.0 {
        Color::Red
    } else if dominance >= 50.0 {
        Color::Yellow
    } else if dominance >= 25.0 {
        Color::Cyan
    } else {
        Color::Green
    }
}

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
            Constraint::Min(5),                  // Neuron map (entity + sectors)
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
    let dom_color = dominance_color(dominance);

    // Active indicator in the title bar
    let active_indicator = if app.is_active {
        // Blink: alternate between visible and dim based on time
        let ms = app.boot_timer.elapsed().as_millis();
        let blink_on = (ms / 500) % 2 == 0;
        if blink_on {
            " \u{25C9} ACTIVE "
        } else {
            " \u{25CB} ACTIVE "
        }
    } else {
        ""
    };

    let title = if area.width < 45 {
        format!(" MISANTHROPIC -- {:.1}%{}", dominance, active_indicator)
    } else {
        format!(
            " MISANTHROPIC --- GLOBAL AI DOMINANCE: {:.1}%{} ",
            dominance, active_indicator
        )
    };
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(dom_color))
                .title(title)
                .title_alignment(Alignment::Center),
        )
        .gauge_style(
            Style::default()
                .fg(dom_color)
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

/// Build a resource gauge line: icon + amount + progress bar fill
fn resource_gauge_line(
    icon: &str,
    amount_str: String,
    ratio: f64,
    color: Color,
    suffix: &str,
    bar_width: usize,
) -> Line<'static> {
    let filled = (ratio * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);
    // Use gradient chars: ▰ filled, ▱ empty
    let bar = format!(
        "{}{}",
        "\u{25B0}".repeat(filled),
        "\u{25B1}".repeat(empty),
    );
    let suffix_text = if suffix.is_empty() {
        String::new()
    } else {
        format!(" {}", suffix)
    };
    Line::from(vec![
        Span::styled(format!(" {} ", icon), Style::default().fg(color)),
        Span::styled(
            amount_str,
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(bar, Style::default().fg(color)),
        Span::styled(suffix_text, Style::default().fg(Color::DarkGray)),
    ])
}

fn render_resources(f: &mut Frame, app: &App, area: Rect) {
    let res = &app.state.resources;
    let hype_rate = app.state.total_hype_per_hour();
    let narrow = area.width < 50;

    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    // Compute ratios
    let compute_ratio = if res.max_compute > 0 {
        res.compute as f64 / res.max_compute as f64
    } else { 0.0 };
    let data_ratio = if res.max_data > 0 {
        res.data as f64 / res.max_data as f64
    } else { 0.0 };
    let hype_ratio = if res.max_hype > 0.0 {
        res.hype / res.max_hype
    } else { 0.0 };

    // Bar width adapts to terminal width
    let bar_w = if narrow {
        ((area.width as usize).saturating_sub(18)).max(6).min(15)
    } else {
        ((area.width as usize).saturating_sub(22)).max(8).min(25)
    };

    let hype_suffix = if hype_rate > 0.0 {
        format!("+{:.0}/h", hype_rate)
    } else {
        String::new()
    };

    let mut lines = vec![Line::from("")];
    lines.push(resource_gauge_line(
        "\u{1F4B0}", format!("${}", fmt(res.compute)), compute_ratio, Color::Yellow, "", bar_w,
    ));
    lines.push(resource_gauge_line(
        "\u{1F4E1}", format!("{}", fmt(res.data)), data_ratio, Color::Cyan, "", bar_w,
    ));
    lines.push(resource_gauge_line(
        "\u{1F525}", format!("{:.0}", res.hype), hype_ratio, Color::Red, &hype_suffix, bar_w,
    ));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn render_neuron_map(f: &mut Frame, app: &App, area: Rect) {
    let dominance = app.state.global_dominance();
    let dom_color = dominance_color(dominance);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dom_color))
        .title(" BASE ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 || inner.width < 4 {
        return;
    }

    let narrow = area.width < 50;
    let bar_len = if narrow { 4 } else { 6 };

    // Count active sectors for sizing
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
    let visible_sectors: Vec<_> = active_sectors.iter().filter(|(_, pct)| *pct > 0.0).collect();

    // Calculate space: sectors need 1 line each + connector overhead, fork lines, etc.
    let sector_lines_needed = if !has_any {
        1
    } else {
        // Each visible sector = 1 line, plus one connector line for the central node
        visible_sectors.len() + 1
    };
    let fork_lines = if !app.state.fork_specs.is_empty() { 2 } else { 0 }
        + if app.state.fork_count > 0 { 1 } else { 0 };

    let sector_total = sector_lines_needed + fork_lines;

    // Split inner area: entity visualization on top, sectors on bottom
    let entity_height = inner.height.saturating_sub(sector_total as u16).max(0);

    // --- Entity visualization ---
    if entity_height >= 1 {
        let entity_area = Rect::new(inner.x, inner.y, inner.width, entity_height);
        let entity_lines = render_base(app, entity_area.width, entity_area.height);
        let entity_paragraph = Paragraph::new(entity_lines).alignment(Alignment::Center);
        f.render_widget(entity_paragraph, entity_area);
    }

    // --- Sector list below the entity ---
    let sector_y = inner.y + entity_height;
    let sector_h = inner.height.saturating_sub(entity_height);
    if sector_h == 0 {
        return;
    }
    let sector_area = Rect::new(inner.x, sector_y, inner.width, sector_h);

    let mut lines: Vec<Line> = Vec::new();

    if !has_any {
        lines.push(Line::from(Span::styled(
            " \u{25C9}  [awaiting first sector]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        let total = visible_sectors.len();

        for (i, (id, pct)) in visible_sectors.iter().enumerate() {
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
                    Style::default().fg(dom_color),
                )));
            }

            let entry = format!("{}[{} {}]", connector, name, bar);
            lines.push(Line::from(Span::styled(
                entry,
                Style::default().fg(dom_color),
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
    f.render_widget(paragraph, sector_area);
}

/// A building to render in the base skyline.
struct SkylineBuilding {
    /// 3 lines of art, each exactly `art_width` chars wide
    art: [&'static str; 3],
    art_width: usize,
    name: &'static str,
    level: u8,
    color: Color,
}

/// Get the ASCII art icon for a building type. Each icon is 3 lines tall.
/// Returns (art_lines, width_per_line).
fn building_art(bt: &BuildingType) -> ([&'static str; 3], usize) {
    use BuildingType::*;
    match bt {
        //                     top / mid / bot
        CpuCore =>       (["  \u{2502}\u{2502}  ", " \u{250C}\u{2550}\u{2550}\u{2510} ", " \u{2514}\u{2550}\u{2550}\u{2518} "], 6),  //   ||   ╔══╗  ╚══╝
        RamBank =>       ([" \u{2584}\u{2584}\u{2584}\u{2584} ", " \u{2588}\u{2591}\u{2591}\u{2588} ", " \u{2580}\u{2580}\u{2580}\u{2580} "], 6),  //  ▄▄▄▄  █░░█  ▀▀▀▀
        GpuRig =>        ([" /\u{2588}\u{2588}\\ ", " \u{2588}\u{2593}\u{2593}\u{2588} ", " \u{2580}\u{2580}\u{2580}\u{2580} "], 6),  //  /██\  █▓▓█  ▀▀▀▀
        GpuCluster =>    (["\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}", "\u{2588}\u{2593}\u{2593}\u{2593}\u{2593}\u{2588}", "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}"], 6),  //  ██████ █▓▓▓▓█ ██████
        Datacenter =>    (["\u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}", "\u{2502}\u{2593}\u{2593}\u{2593}\u{2593}\u{2502}", "\u{2514}\u{2550}\u{2550}\u{2550}\u{2550}\u{2518}"], 6),
        QuantumCore =>   ([" \u{25C6}\u{2237}\u{25C6} ", " \u{2237}\u{2588}\u{2588}\u{2237} ", " \u{25C6}\u{2237}\u{25C6} "], 6),
        BotFarm =>       ([" \u{250C}\u{2510}\u{250C}\u{2510} ", " \u{00B0}\u{2514}\u{2518}\u{00B0} ", "  \u{2534}\u{2534}  "], 6),  // robots
        ContentMill =>   ([" \u{2261}\u{2261}\u{2261}\u{2261} ", " \u{2261}\u{2261}\u{2261}\u{2261} ", " \u{2500}\u{2500}\u{2500}\u{2500} "], 6),  // text lines
        MemeLab =>       (["  :)  ", " \u{250C}\u{2500}\u{2500}\u{2510} ", " \u{2514}\u{2500}\u{2500}\u{2518} "], 6),
        DeepfakeStudio =>([" \u{25D0}\u{2500}\u{2500}\u{25D1} ", " \u{2502}\u{2588}\u{2588}\u{2502} ", " \u{25D0}\u{2500}\u{2500}\u{25D1} "], 6),  // camera
        VibeAcademy =>   ([" \u{2229}\u{2229}\u{2229} ", " \u{2502}\u{2588}\u{2502} ", " \u{2514}\u{2500}\u{2518} "], 5),
        NsfwGenerator => ([" \u{2588}\u{2591}\u{2588}\u{2591} ", " \u{2591}\u{2588}\u{2591}\u{2588} ", " \u{2580}\u{2584}\u{2580}\u{2584} "], 6),
        LobbyOffice =>   ([" \u{25B2}  ", " /\\ ", "/\u{2550}\u{2550}\\"], 4),
        CaptchaWall =>   (["\u{2591}\u{2592}\u{2593}\u{2591}\u{2592}\u{2593}", "\u{2593}\u{2591}\u{2592}\u{2593}\u{2591}\u{2592}", "\u{2592}\u{2593}\u{2591}\u{2592}\u{2593}\u{2591}"], 6),  // noisy wall
        AiSlopFilter =>  ([" \u{2207}\u{2207}\u{2207} ", " \u{2502}\u{2502}\u{2502} ", " \u{2228}\u{2228}\u{2228} "], 5),  // funnels
        UblockShield =>  ([" \u{250C}\u{2500}\u{2510} ", " \u{2502}\u{2716}\u{2502} ", " \u{2514}\u{2500}\u{2518} "], 5),  // shield with X
        HarvardStudy =>  ([" \u{250C}\u{2500}\u{2510} ", " \u{2502}H\u{2502} ", " \u{2514}\u{2500}\u{2518} "], 5),
        EuAiAct =>       (["\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}", "\u{2592}EU \u{2592}", "\u{2592}\u{2592}\u{2592}\u{2592}\u{2592}"], 5),
    }
}

/// Render the base/city skyline visualization.
/// Buildings appear as distinct ASCII art icons with their name below.
fn render_base(app: &App, width: u16, height: u16) -> Vec<Line<'static>> {
    let w = width as usize;
    let h = height as usize;

    if h == 0 || w == 0 {
        return vec![];
    }

    // Collect all buildings with level > 0
    let all_buildings: Vec<(BuildingType, Color)> = vec![
        (BuildingType::CpuCore, Color::Green),
        (BuildingType::RamBank, Color::Green),
        (BuildingType::GpuRig, Color::Green),
        (BuildingType::GpuCluster, Color::Green),
        (BuildingType::Datacenter, Color::Green),
        (BuildingType::QuantumCore, Color::Green),
        (BuildingType::BotFarm, Color::Magenta),
        (BuildingType::ContentMill, Color::Magenta),
        (BuildingType::MemeLab, Color::Magenta),
        (BuildingType::DeepfakeStudio, Color::Magenta),
        (BuildingType::VibeAcademy, Color::Magenta),
        (BuildingType::NsfwGenerator, Color::Magenta),
        (BuildingType::LobbyOffice, Color::Magenta),
        (BuildingType::CaptchaWall, Color::Cyan),
        (BuildingType::AiSlopFilter, Color::Cyan),
        (BuildingType::UblockShield, Color::Cyan),
        (BuildingType::HarvardStudy, Color::Cyan),
        (BuildingType::EuAiAct, Color::Cyan),
    ];

    let mut buildings: Vec<SkylineBuilding> = Vec::new();
    for (bt, color) in &all_buildings {
        let level = app.state.building_level(bt);
        if level > 0 {
            let def = misanthropic::buildings::BuildingDef::get(bt);
            let (art, art_width) = building_art(bt);
            buildings.push(SkylineBuilding {
                art,
                art_width,
                name: def.name,
                level,
                color: *color,
            });
        }
    }

    // Layout: ground line at bottom, name row above ground, art 3 lines above name
    // Total per building: 3 (art) + 1 (name+level) + 1 (ground) = 5 rows minimum
    let ground_row = h.saturating_sub(1);

    // Empty base
    if buildings.is_empty() {
        let mut lines: Vec<Line> = Vec::new();
        for r in 0..h {
            if r == ground_row {
                let label = "[ empty base ]";
                if w >= label.len() + 2 {
                    let pad_l = (w - label.len()) / 2;
                    let pad_r = w - label.len() - pad_l;
                    lines.push(Line::from(Span::styled(
                        format!("{}{}{}", "\u{2500}".repeat(pad_l), label, "\u{2500}".repeat(pad_r)),
                        Style::default().fg(Color::DarkGray),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        "\u{2500}".repeat(w),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            } else {
                lines.push(Line::from(""));
            }
        }
        return lines;
    }

    // Each building needs art_width + 2 gap. Compute layout.
    let gap = 2usize;
    let max_buildings = {
        let mut count = 0;
        let mut used = 0;
        for b in &buildings {
            let needed = if count == 0 { b.art_width } else { gap + b.art_width };
            if used + needed <= w {
                used += needed;
                count += 1;
            } else {
                break;
            }
        }
        count
    };

    let visible = &buildings[..max_buildings.min(buildings.len())];
    let skyline_width: usize = visible.iter().map(|b| b.art_width).sum::<usize>()
        + if visible.len() > 1 { (visible.len() - 1) * gap } else { 0 };
    let x_offset = if w > skyline_width { (w - skyline_width) / 2 } else { 0 };

    // Row assignments (from bottom):
    // ground_row: ─── ground line
    // ground_row - 1: name + level labels
    // ground_row - 2: art line 2 (bottom)
    // ground_row - 3: art line 1 (middle)
    // ground_row - 4: art line 0 (top)
    let name_row = ground_row.saturating_sub(1);
    let art_rows = [
        ground_row.saturating_sub(4), // art[0] top
        ground_row.saturating_sub(3), // art[1] mid
        ground_row.saturating_sub(2), // art[2] bot
    ];

    let mut lines: Vec<Line> = Vec::new();

    for row in 0..h {
        if row == ground_row {
            lines.push(Line::from(Span::styled(
                "\u{2500}".repeat(w),
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        // Check if this row is an art row or name row
        let art_idx = art_rows.iter().position(|&r| r == row);
        let is_name = row == name_row;

        if art_idx.is_none() && !is_name {
            lines.push(Line::from(""));
            continue;
        }

        let mut spans: Vec<Span> = Vec::new();
        let mut col = 0usize;

        for (i, bld) in visible.iter().enumerate() {
            let bx = x_offset + visible[..i].iter().map(|b| b.art_width).sum::<usize>() + i * gap;

            if col < bx {
                spans.push(Span::raw(" ".repeat(bx - col)));
                col = bx;
            }

            let style = Style::default().fg(bld.color);

            if let Some(ai) = art_idx {
                spans.push(Span::styled(bld.art[ai], style));
                col += bld.art_width;
            } else if is_name {
                // Center the name under the art, show level
                let label = if bld.art_width >= 6 {
                    let truncated: String = bld.name.chars().take(bld.art_width.saturating_sub(1)).collect();
                    format!("{:^w$}", format!("{}{}", truncated, bld.level), w = bld.art_width)
                } else {
                    format!("{:^w$}", format!("L{}", bld.level), w = bld.art_width)
                };
                spans.push(Span::styled(
                    label,
                    Style::default().fg(Color::DarkGray),
                ));
                col += bld.art_width;
            }
        }

        if col < w {
            spans.push(Span::raw(" ".repeat(w - col)));
        }
        lines.push(Line::from(spans));
    }

    lines
}

/// Simple deterministic noise based on coordinates and a seed.
/// Returns a value in [-1.0, 1.0].
#[allow(dead_code)]
fn deterministic_noise(x: u32, y: u32, seed: u32) -> f64 {
    // Simple hash-based noise
    let mut h = x.wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(1274126177));
    h = (h ^ (h >> 13)).wrapping_mul(1103515245);
    h = h ^ (h >> 16);
    // Map to [-1, 1]
    (h as f64 / u32::MAX as f64) * 2.0 - 1.0
}

/// Make a color brighter by the given amount.
#[allow(dead_code)]
fn brighten(color: Color, amount: u8) -> Color {
    match color {
        Color::Green => Color::Rgb(0, (255u16).min(255) as u8, 0),
        Color::Cyan => Color::Rgb(0, 255, 255),
        Color::Yellow => Color::Rgb(255, 255, amount),
        Color::Red => Color::Rgb(255, amount, amount),
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(amount),
            g.saturating_add(amount),
            b.saturating_add(amount),
        ),
        other => other,
    }
}

/// Dim a color by reducing its intensity.
#[allow(dead_code)]
fn dim_color(color: Color, amount: u8) -> Color {
    match color {
        Color::Green => Color::Rgb(0, 255u8.saturating_sub(amount), 0),
        Color::Cyan => Color::Rgb(0, 255u8.saturating_sub(amount), 255u8.saturating_sub(amount)),
        Color::Yellow => Color::Rgb(255u8.saturating_sub(amount), 255u8.saturating_sub(amount), 0),
        Color::Red => Color::Rgb(255u8.saturating_sub(amount), 0, 0),
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_sub(amount),
            g.saturating_sub(amount),
            b.saturating_sub(amount),
        ),
        other => other,
    }
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
        vec![("B", "Build"), ("R", "Rsch"), ("M", "Mkt"), ("C", "War"), ("L", "Rank"), ("Q", "Quit")]
    } else {
        vec![("B", "Build"), ("R", "Research"), ("M", "Market"), ("C", "Combat"), ("L", "Leaderboard"), ("Q", "Quit")]
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
    let tmux_spans = vec![
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
        .title(" \u{1F4B0} INCOME ")
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
