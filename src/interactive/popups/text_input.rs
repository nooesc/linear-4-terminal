use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::interactive::app::{InteractiveApp, Popup, TextInputContext};
use crate::interactive::layout::centered_popup;

/// Draw a text input popup for comments, search, title edit, description edit, or filter.
pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let Some(Popup::TextInput(ctx)) = &app.popup else {
        return;
    };

    let (title, hints) = match ctx {
        TextInputContext::Comment => (
            "Add Comment",
            "Enter: Submit  Esc: Cancel",
        ),
        TextInputContext::Search => (
            "Search",
            "Enter: Apply  Esc: Cancel",
        ),
        TextInputContext::EditTitle => (
            "Edit Title",
            "Enter: Submit  Esc: Cancel",
        ),
        TextInputContext::EditDescription => (
            "Edit Description",
            "Enter: Submit  Ctrl+E: External Editor  Esc: Cancel",
        ),
        TextInputContext::Filter => (
            "Filter",
            "Enter: Apply  Esc: Cancel",
        ),
    };

    let width: u16 = 60;
    let height: u16 = 5;
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Build the input text with a cursor indicator
    let input = &app.text_input;
    let cursor_pos = app.text_cursor.min(input.len());

    let (before, after) = input.split_at(cursor_pos);
    let cursor_char = after.chars().next().unwrap_or(' ');
    let rest = if after.len() > cursor_char.len_utf8() {
        &after[cursor_char.len_utf8()..]
    } else {
        ""
    };

    let input_line = Line::from(vec![
        Span::styled(before, Style::default().fg(Color::White)),
        Span::styled(
            cursor_char.to_string(),
            Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(rest, Style::default().fg(Color::White)),
    ]);

    // Render input on first line of inner area
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let input_widget = Paragraph::new(input_line);
    frame.render_widget(input_widget, input_area);

    // Render hints on the last line
    let hints_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    let hints_widget = Paragraph::new(Line::from(Span::styled(
        hints,
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hints_widget, hints_area);
}
