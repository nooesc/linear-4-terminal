use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::interactive::app::{Focus, InteractiveApp};

pub fn draw_teams(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::TeamList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(" Teams ({}) ", app.teams.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if app.teams.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No teams")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll_offset = if app.team_index >= inner_height {
        app.team_index - inner_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .teams
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|(i, team)| {
            let marker = if app.active_team == Some(i) { "►" } else { " " };
            let display = format!("{} {} ({})", marker, team.name, team.key);

            let style = if i == app.team_index && focused {
                Style::default()
                    .bg(Color::Rgb(30, 35, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if app.active_team == Some(i) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
