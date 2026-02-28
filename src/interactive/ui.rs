use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::interactive::app::InteractiveApp;
use crate::interactive::layout;

pub fn draw(frame: &mut Frame, app: &InteractiveApp) {
    let area = frame.size();

    // Calculate layout
    let active_notifs = app.notifications.iter().filter(|n| !n.dismissed).count();
    let app_layout = layout::app_layout(area, active_notifs);
    let panels = layout::panel_layout(app_layout.main);

    // Header
    super::panels::header::draw_header(frame, app_layout.header, app);

    // Left panel: issue list
    super::panels::list::draw_list(frame, panels.left, app);

    // Right panel: detail (only in two-panel mode)
    if panels.right.width > 0 {
        super::panels::detail::draw_detail(frame, panels.right, app);
    }

    // Notifications
    if app_layout.notifications.height > 0 {
        super::notifications::draw(frame, app_layout.notifications, app);
    }

    // Footer
    draw_footer(frame, app_layout.footer, app);

    // Popup overlay (drawn last, on top of everything)
    super::popups::draw_popup(frame, area, app);
}

fn draw_footer(frame: &mut Frame, area: ratatui::layout::Rect, app: &InteractiveApp) {
    let help_text = if app.popup.is_some() {
        "" // Popup has its own hints
    } else if !app.multi_selected.is_empty() {
        "[Space] Bulk action  [x] Toggle select  [X] Clear  [Esc] Cancel"
    } else {
        "[s]tatus [c]omment [l]abels [p]roject [a]ssign [e]dit [n]ew [/]search [f]ilter [?]help"
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::LightGreen))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}
