use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::interactive::app::InteractiveApp;
use crate::interactive::layout::centered_popup;

/// Draw the full keyboard shortcuts help overlay.
pub fn draw(frame: &mut Frame, area: Rect, _app: &InteractiveApp) {
    let width: u16 = 70;
    let height: u16 = 22;
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Keyboard Shortcuts ")
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let header_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let separator_style = Style::default().fg(Color::DarkGray);
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);

    // Build the three-column layout as lines
    // Each line contains content across all three columns
    let lines: Vec<Line> = vec![
        // Column headers
        Line::from(vec![
            Span::styled(format!("{:<20}", "Navigation"), header_style),
            Span::styled(format!("{:<21}", "Actions"), header_style),
            Span::styled("Panels", header_style),
        ]),
        // Separators
        Line::from(vec![
            Span::styled(
                format!("{:<20}", "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"),
                separator_style,
            ),
            Span::styled(
                format!("{:<21}", "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}"),
                separator_style,
            ),
            Span::styled(
                "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                separator_style,
            ),
        ]),
        // Row 1
        build_help_row("j/k", "Move up/down", "s", "Change status", "Tab", "Switch focus", key_style, desc_style),
        // Row 2
        build_help_row("g", "Group by", "c", "Add comment", "?", "This help", key_style, desc_style),
        // Row 3
        build_help_row("/", "Search", "l", "Change labels", "Esc", "Back/close", key_style, desc_style),
        // Row 4
        build_help_row("f", "Filter", "p", "Change project", "q", "Quit", key_style, desc_style),
        // Row 5
        build_help_row("d", "Toggle done", "a", "Change assignee", "", "", key_style, desc_style),
        // Row 6
        build_help_row("r", "Refresh", "e", "Full edit", "", "", key_style, desc_style),
        // Row 7
        build_help_row("n", "New issue", "o", "Open in browser", "", "", key_style, desc_style),
        // Row 8
        build_help_row("x", "Multi-select", "", "", "", "", key_style, desc_style),
        // Row 9
        build_help_row("X", "Clear selection", "", "", "", "", key_style, desc_style),
        // Row 10
        build_help_row("Space", "Bulk actions", "", "", "", "", key_style, desc_style),
    ];

    let content = Paragraph::new(lines);
    let content_area = Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), inner.height.saturating_sub(1));
    frame.render_widget(content, content_area);

    // Footer
    let footer_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    let footer = Paragraph::new(Line::from(Span::styled(
        "Press ? or Esc to close",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(footer, footer_area);
}

/// Build a single row across three columns (Navigation, Actions, Panels).
fn build_help_row<'a>(
    nav_key: &'a str,
    nav_desc: &'a str,
    act_key: &'a str,
    act_desc: &'a str,
    pan_key: &'a str,
    pan_desc: &'a str,
    key_style: Style,
    desc_style: Style,
) -> Line<'a> {
    let mut spans = Vec::new();

    // Navigation column (width 20)
    if nav_key.is_empty() {
        spans.push(Span::styled(format!("{:<20}", ""), desc_style));
    } else {
        spans.push(Span::styled(format!("{:<6}", nav_key), key_style));
        let desc_with_pad = format!("{:<14}", nav_desc);
        spans.push(Span::styled(desc_with_pad, desc_style));
    }

    // Actions column (width 21)
    if act_key.is_empty() {
        spans.push(Span::styled(format!("{:<21}", ""), desc_style));
    } else {
        spans.push(Span::styled(format!("{:<3}", act_key), key_style));
        let desc_with_pad = format!("{:<18}", act_desc);
        spans.push(Span::styled(desc_with_pad, desc_style));
    }

    // Panels column
    if pan_key.is_empty() {
        // nothing
    } else {
        spans.push(Span::styled(format!("{:<5}", pan_key), key_style));
        spans.push(Span::styled(pan_desc, desc_style));
    }

    Line::from(spans)
}
