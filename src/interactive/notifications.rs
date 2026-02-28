use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::interactive::app::{InteractiveApp, NotificationKind};

pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    if app.notifications.is_empty() || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = app.notifications.iter()
        .filter(|n| !n.dismissed)
        .take(3)
        .map(|n| {
            let (icon, color) = match n.kind {
                NotificationKind::Success => ("✓", Color::Green),
                NotificationKind::Error => ("✗", Color::Red),
                NotificationKind::Loading => ("⟳", Color::Yellow),
                NotificationKind::Info => ("ⓘ", Color::Blue),
            };
            let elapsed = n.created_at.elapsed().as_secs();
            let timer = match n.kind {
                NotificationKind::Success | NotificationKind::Info => {
                    let remaining = 5u64.saturating_sub(elapsed);
                    format!("[{}s]", remaining)
                }
                _ => String::new(),
            };
            Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(n.message.clone(), Style::default().fg(color)),
                Span::styled(format!("  {}", timer), Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
