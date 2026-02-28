use std::collections::HashMap;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::interactive::app::{Focus, InteractiveApp};

pub fn draw_projects(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::ProjectList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Count issues per project from loaded issues
    let mut project_counts: HashMap<&str, usize> = HashMap::new();
    let mut total_issues = 0usize;
    for issue in &app.issues {
        total_issues += 1;
        if let Some(ref proj) = issue.project {
            *project_counts.entry(&proj.id).or_insert(0) += 1;
        }
    }

    let title = format!(" Projects ({}) ", app.available_projects.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll_offset = if app.project_index >= inner_height {
        app.project_index - inner_height + 1
    } else {
        0
    };

    // Build options: "All" at index 0, then each project
    let mut options: Vec<(usize, String)> = vec![(0, format!("All ({})", total_issues))];
    options.extend(
        app.available_projects
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let count = project_counts.get(p.id.as_str()).copied().unwrap_or(0);
                (i + 1, format!("{} ({})", p.name, count))
            }),
    );

    let items: Vec<ListItem> = options
        .iter()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|(idx, name)| {
            let is_active = match app.active_project {
                None => *idx == 0,     // None means "All" is active
                Some(ap) => ap == *idx,
            };
            let marker = if is_active { "►" } else { " " };
            let display = format!("{} {}", marker, name);

            let style = if *idx == app.project_index && focused {
                Style::default()
                    .bg(Color::Rgb(30, 35, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_active {
                Style::default().fg(Color::LightGreen)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
