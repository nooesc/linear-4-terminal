use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::interactive::app::InteractiveApp;
use crate::interactive::layout::centered_popup;

/// Bulk action options
const BULK_OPTIONS: &[&str] = &[
    "Change status",
    "Change priority",
    "Change project",
    "Add labels",
    "Change assignee",
    "Archive",
];

/// Draw the bulk actions menu popup.
pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let count = app.multi_selected.len();

    let width: u16 = 30;
    let height: u16 = 10;
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Bulk Actions ({} issues) ", count))
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let items: Vec<ListItem> = BULK_OPTIONS
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == app.picker_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!(" {} ", option), style)))
        })
        .collect();

    let list_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    let list = List::new(items);
    frame.render_widget(list, list_area);

    // Hints at the bottom
    let hints_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    let hints_widget = Paragraph::new(Line::from(Span::styled(
        "Enter: Select  Esc: Cancel",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hints_widget, hints_area);
}
