use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::list::truncate;
use crate::interactive::app::{GroupBy, InteractiveApp};

pub fn draw_header(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    // Left side: selected issue identifier + title, or fallback
    let title = if let Some(issue) = app.get_selected_issue() {
        let max_title_len = (header_chunks[0].width as usize)
            .saturating_sub(issue.identifier.len() + 6);
        format!(
            " {} - {} ",
            issue.identifier,
            truncate(&issue.title, max_title_len)
        )
    } else {
        " Linear CLI ".to_string()
    };

    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(header, header_chunks[0]);

    // Right side: issue count, group-by mode, hide-done, active filter
    let done_text = if app.hide_done_issues {
        " | Done: Hidden"
    } else {
        ""
    };

    let filter_text = if !app.filter_query.is_empty() {
        format!(" | Filter: {}", truncate(&app.filter_query, 15))
    } else {
        String::new()
    };

    let info = format!(
        " Issues: {} | Group: {}{}{}",
        app.filtered_issues.len(),
        match app.group_by {
            GroupBy::Status => "Status",
            GroupBy::Project => "Project",
        },
        done_text,
        filter_text,
    );

    let info_widget = Paragraph::new(info)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(info_widget, header_chunks[1]);
}
