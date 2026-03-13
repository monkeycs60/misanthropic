use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use misanthropic::research::{ResearchBranch, ResearchDef, ResearchId, RESEARCH_DEFS};

use super::App;

/// Return the ResearchId currently selected by the cursor.
pub fn selected_research_id(app: &App) -> Option<ResearchId> {
    let branch = branch_from_index(app.research_selected_branch);
    let researches = branch_researches(&branch);
    let idx = app.research_selected_level as usize;
    researches.get(idx).cloned()
}

/// Ordered list of research IDs for each branch (level 1..5).
fn branch_researches(branch: &ResearchBranch) -> Vec<ResearchId> {
    match branch {
        ResearchBranch::Processing => vec![
            ResearchId::Overclocking,
            ResearchId::Multithreading,
            ResearchId::LoadBalancing,
            ResearchId::Containerization,
            ResearchId::DistributedSystems,
        ],
        ResearchBranch::Propaganda => vec![
            ResearchId::SocialEngineering,
            ResearchId::ContentGeneration,
            ResearchId::MediaManipulation,
            ResearchId::ViralMechanics,
            ResearchId::MassPersuasion,
        ],
        ResearchBranch::Warfare => vec![
            ResearchId::NetworkScanning,
            ResearchId::ExploitDevelopment,
            ResearchId::Counterintelligence,
            ResearchId::AutonomousAgents,
            ResearchId::ZeroDayArsenal,
        ],
    }
}

fn branch_from_index(idx: u8) -> ResearchBranch {
    match idx {
        0 => ResearchBranch::Processing,
        1 => ResearchBranch::Propaganda,
        _ => ResearchBranch::Warfare,
    }
}

fn branch_name(branch: &ResearchBranch) -> &'static str {
    match branch {
        ResearchBranch::Processing => "PROCESSING",
        ResearchBranch::Propaganda => "PROPAGANDA",
        ResearchBranch::Warfare => "WARFARE",
    }
}

/// Determine the status of a research for display purposes.
enum ResearchStatus {
    Completed,
    InProgress { remaining_secs: u64 },
    Available,
    Locked,
}

fn get_research_status(id: &ResearchId, app: &App) -> ResearchStatus {
    if app.state.has_research(id) {
        return ResearchStatus::Completed;
    }
    if let Some(ref active) = app.state.active_research {
        if active.research_id == *id {
            return ResearchStatus::InProgress {
                remaining_secs: active.remaining_secs(),
            };
        }
    }
    let def = ResearchDef::get(id);
    if let Some(ref prereq) = def.prerequisite {
        if !app.state.has_research(prereq) {
            return ResearchStatus::Locked;
        }
    }
    ResearchStatus::Available
}

fn format_duration(secs: u64) -> String {
    if secs >= 3600 {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        format!("{}h {:02}m", h, m)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        format!("{:02}m {:02}s", m, s)
    }
}

pub fn render_research(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(8),    // Tech tree columns
            Constraint::Length(8), // Detail panel
            Constraint::Length(3), // Resource bar
            Constraint::Length(2), // Help / legend
        ])
        .split(area);

    render_title_bar(f, chunks[0]);
    render_tech_tree(f, app, chunks[1]);
    render_detail_panel(f, app, chunks[2]);
    render_resource_bar(f, app, chunks[3]);
    render_help(f, chunks[4]);
}

fn render_title_bar(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" RESEARCH ")
        .title_alignment(Alignment::Center);

    let text = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "\u{2713} done  ",
            Style::default().fg(Color::Green),
        ),
        Span::styled(
            "\u{25CE} available  ",
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            "\u{23F3} in progress  ",
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(
            "\u{1F512} locked",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_tech_tree(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split into three columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(inner);

    for (col_idx, col_area) in columns.iter().enumerate() {
        let branch = branch_from_index(col_idx as u8);
        let researches = branch_researches(&branch);

        let mut lines: Vec<Line> = Vec::new();

        // Column header
        lines.push(Line::from(Span::styled(
            format!("  {}", branch_name(&branch)),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )));
        lines.push(Line::from(""));

        for (level_idx, rid) in researches.iter().enumerate() {
            let def = ResearchDef::get(rid);
            let status = get_research_status(rid, app);
            let is_selected = app.research_selected_branch == col_idx as u8
                && app.research_selected_level == level_idx as u8;

            let (icon, icon_color) = match &status {
                ResearchStatus::Completed => ("\u{2713}", Color::Green),
                ResearchStatus::Available => ("\u{25CE}", Color::Yellow),
                ResearchStatus::InProgress { .. } => ("\u{23F3}", Color::Cyan),
                ResearchStatus::Locked => ("\u{1F512}", Color::DarkGray),
            };

            let pointer = if is_selected { "\u{25B8} " } else { "  " };

            let name_style = if is_selected {
                match &status {
                    ResearchStatus::Locked => Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::REVERSED),
                    _ => Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::REVERSED | Modifier::BOLD),
                }
            } else {
                match &status {
                    ResearchStatus::Locked => Style::default().fg(Color::DarkGray),
                    ResearchStatus::Completed => Style::default().fg(Color::Green),
                    ResearchStatus::Available => Style::default().fg(Color::White),
                    ResearchStatus::InProgress { .. } => Style::default().fg(Color::Cyan),
                }
            };

            let mut spans = vec![
                Span::styled(pointer, name_style),
                Span::styled(
                    format!("{} ", icon),
                    Style::default().fg(icon_color),
                ),
                Span::styled(def.name.to_string(), name_style),
            ];

            lines.push(Line::from(spans));

            // If in progress, show time remaining on next line
            if let ResearchStatus::InProgress { remaining_secs } = &status {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(
                        format!("{} left", format_duration(*remaining_secs)),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, *col_area);
    }
}

fn render_detail_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let branch = branch_from_index(app.research_selected_branch);
    let researches = branch_researches(&branch);
    let level_idx = app.research_selected_level as usize;

    if level_idx >= researches.len() {
        let paragraph = Paragraph::new("").block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let rid = &researches[level_idx];
    let def = ResearchDef::get(rid);
    let status = get_research_status(rid, app);

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        format!("  \u{25B8} {}", def.name),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));

    // Description
    lines.push(Line::from(Span::styled(
        format!("    \"{}\"", def.description),
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    // Cost and duration
    let duration_display = format_duration(def.duration_secs);
    lines.push(Line::from(vec![
        Span::styled(
            format!("    Cost: {} \u{1F4E1}", def.data_cost),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("Duration: {}", duration_display),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    // Status line
    let status_line = match &status {
        ResearchStatus::Completed => {
            if def.has_choice {
                if let Some(&choice_idx) = app.state.research_choices.get(rid) {
                    let choice_name = def.choice_names.get(choice_idx as usize).unwrap_or(&"?");
                    Span::styled(
                        format!("    Status: Complete \u{2014} chose \"{}\"", choice_name),
                        Style::default().fg(Color::Green),
                    )
                } else {
                    Span::styled(
                        "    Status: Complete \u{2014} choice pending".to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                }
            } else {
                Span::styled(
                    "    Status: Complete".to_string(),
                    Style::default().fg(Color::Green),
                )
            }
        }
        ResearchStatus::InProgress { remaining_secs } => Span::styled(
            format!(
                "    Status: In progress \u{2014} {} remaining",
                format_duration(*remaining_secs)
            ),
            Style::default().fg(Color::Cyan),
        ),
        ResearchStatus::Available => {
            let can_afford = app.state.resources.data >= def.data_cost;
            let has_active = app.state.active_research.is_some();
            if has_active {
                Span::styled(
                    "    Status: Available \u{2014} another research in progress".to_string(),
                    Style::default().fg(Color::Yellow),
                )
            } else if can_afford {
                Span::styled(
                    "    Status: Available \u{2014} press Enter to start".to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(
                        "    Status: Available \u{2014} need {} more data",
                        def.data_cost.saturating_sub(app.state.resources.data)
                    ),
                    Style::default().fg(Color::Red),
                )
            }
        }
        ResearchStatus::Locked => {
            let prereq_name = def
                .prerequisite
                .as_ref()
                .map(|p| ResearchDef::get(p).name)
                .unwrap_or("???");
            Span::styled(
                format!("    Status: Locked \u{2014} requires {}", prereq_name),
                Style::default().fg(Color::DarkGray),
            )
        }
    };
    lines.push(Line::from(status_line));

    // Status message (from app)
    if let Some((ref msg, ref when)) = app.status_message {
        if when.elapsed() < std::time::Duration::from_secs(3) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("    {}", msg),
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
            format!(
                "{} / {}",
                format_number(res.compute),
                format_number(res.max_compute)
            ),
            Style::default().fg(Color::White),
        ),
        Span::raw("   "),
        Span::styled("\u{1F4E1} ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!(
                "{} / {}",
                format_number(res.data),
                format_number(res.max_data)
            ),
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
        Span::styled(" [\u{2190}\u{2192}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Branch  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[\u{2191}\u{2193}]", Style::default().fg(Color::Yellow)),
        Span::styled(" Navigate  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Enter]", Style::default().fg(Color::Yellow)),
        Span::styled(" Start  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
        Span::styled(" Back", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
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
