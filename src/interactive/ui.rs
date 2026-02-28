use ratatui::{
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};
use crate::interactive::app::InteractiveApp;
use crate::interactive::layout;

pub fn draw(frame: &mut Frame, app: &InteractiveApp) {
    let area = frame.size();

    // Guard: terminal too small to render anything meaningful
    if area.width < 20 || area.height < 5 {
        let msg = ratatui::widgets::Paragraph::new("Terminal too small")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    // Calculate layout
    let active_notifs = app.notifications.iter().filter(|n| !n.dismissed).count();
    let app_layout = layout::app_layout(area, active_notifs);
    let panels = layout::panel_layout(app_layout.main);

    // Header
    if app_layout.header.height > 0 {
        super::panels::header::draw_header(frame, app_layout.header, app);
    }

    let two_panel = panels.right.width > 0;
    if two_panel {
        // Two-panel mode: always show both left column and detail
        let left_col = layout::left_column_layout(
            panels.left,
            app.teams.len(),
            app.available_projects.len() + 1,
        );
        super::panels::teams::draw_teams(frame, left_col.teams, app);
        super::panels::projects::draw_projects(frame, left_col.projects, app);
        super::panels::list::draw_list(frame, left_col.issues, app);
        super::panels::detail::draw_detail(frame, panels.right, app);
    } else if app.show_detail_fullscreen {
        // Single-panel mode, detail view: show detail full-width
        super::panels::detail::draw_detail(frame, panels.left, app);
    } else {
        // Single-panel mode, list view: show left column full-width
        let left_col = layout::left_column_layout(
            panels.left,
            app.teams.len(),
            app.available_projects.len() + 1,
        );
        super::panels::teams::draw_teams(frame, left_col.teams, app);
        super::panels::projects::draw_projects(frame, left_col.projects, app);
        super::panels::list::draw_list(frame, left_col.issues, app);
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
    use ratatui::text::{Line, Span};

    let help_text = if app.popup.is_some() {
        "" // Popup has its own hints
    } else if !app.multi_selected.is_empty() {
        "[Space] Bulk  [x] Toggle  [X] Clear  [Esc] Cancel"
    } else {
        " s:status c:comment l:labels p:project a:assign e:edit n:new /:search f:filter ?:help"
    };

    let line = Line::from(vec![
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
    ]);

    let footer = Paragraph::new(line)
        .style(Style::default().bg(Color::Rgb(20, 22, 30)));
    frame.render_widget(footer, area);
}
