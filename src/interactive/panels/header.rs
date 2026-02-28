use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::list::truncate;
use crate::interactive::app::{GroupBy, InteractiveApp};

pub fn draw_header(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let width = area.width as usize;

    // Left: selected issue info
    let left = if let Some(issue) = app.get_selected_issue() {
        let max_len = width / 2;
        let title = truncate(&issue.title, max_len.saturating_sub(issue.identifier.len() + 3));
        vec![
            Span::styled(
                format!(" {} ", issue.identifier),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(title, Style::default().fg(Color::White)),
        ]
    } else {
        vec![Span::styled(
            " Linear CLI",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]
    };

    // Right: status indicators
    let mut right_parts = Vec::new();

    let group_label = match app.group_by {
        GroupBy::Status => "status",
        GroupBy::Project => "project",
    };
    right_parts.push(Span::styled(
        format!("group:{}", group_label),
        Style::default().fg(Color::DarkGray),
    ));

    if app.hide_done_issues {
        right_parts.push(Span::styled(" hide:done", Style::default().fg(Color::DarkGray)));
    }

    if !app.filter_query.is_empty() {
        right_parts.push(Span::styled(
            format!(" filter:{}", truncate(&app.filter_query, 12)),
            Style::default().fg(Color::Yellow),
        ));
    }

    right_parts.push(Span::raw(" "));

    // Calculate right side width to pad correctly
    let right_text_len: usize = right_parts.iter().map(|s| s.content.len()).sum();
    let left_text_len: usize = left.iter().map(|s| s.content.len()).sum();
    let pad = width.saturating_sub(left_text_len + right_text_len);

    let mut spans = left;
    spans.push(Span::raw(" ".repeat(pad)));
    spans.extend(right_parts);

    let header = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Rgb(20, 22, 30)));
    frame.render_widget(header, area);
}
