use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::interactive::app::{ConfirmAction, InteractiveApp, Popup};
use crate::interactive::layout::centered_popup;

/// Draw a small confirmation dialog.
pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let Some(Popup::Confirmation(action)) = &app.popup else {
        return;
    };

    let message = match action {
        ConfirmAction::ArchiveIssue(_) => "Archive this issue?",
    };

    let width: u16 = 40;
    let height: u16 = 5;
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Message line
    let message_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let message_widget = Paragraph::new(Line::from(Span::styled(
        message,
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    frame.render_widget(message_widget, message_area);

    // Options line: [Y]es  [N]o
    let options_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    let options_line = Line::from(vec![
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "Y",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("]es  ", Style::default().fg(Color::DarkGray)),
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "N",
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("]o", Style::default().fg(Color::DarkGray)),
    ]);
    let options_widget = Paragraph::new(options_line);
    frame.render_widget(options_widget, options_area);
}
