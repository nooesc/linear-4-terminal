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

    // +1 for the "All" option
    let count = app.available_projects.len() + 1;
    let title = format!(" Projects ({}) ", count);
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
    let mut options: Vec<(usize, String)> = vec![(0, "All".to_string())];
    options.extend(
        app.available_projects
            .iter()
            .enumerate()
            .map(|(i, p)| (i + 1, p.name.clone())),
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
