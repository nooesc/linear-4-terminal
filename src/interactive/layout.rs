use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Top-level layout regions
pub struct AppLayout {
    pub header: Rect,
    pub main: Rect,
    pub notifications: Rect,
    pub footer: Rect,
}

/// Panel split within the main area
pub struct PanelLayout {
    pub left: Rect,
    pub right: Rect,
}

/// Calculate the top-level layout
pub fn app_layout(area: Rect, notification_count: usize) -> AppLayout {
    let notif_height = if notification_count > 0 {
        (notification_count as u16).min(3) + 2
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(notif_height),
            Constraint::Length(3),
        ])
        .split(area);

    AppLayout {
        header: chunks[0],
        main: chunks[1],
        notifications: chunks[2],
        footer: chunks[3],
    }
}

/// Split main area into left (issue list) and right (detail) panels.
/// On narrow terminals (<100 cols), returns full width for left, zero for right.
pub fn panel_layout(area: Rect) -> PanelLayout {
    if area.width < 100 {
        PanelLayout {
            left: area,
            right: Rect::default(),
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area);

        PanelLayout {
            left: chunks[0],
            right: chunks[1],
        }
    }
}

/// Whether we're in single-panel mode (narrow terminal)
pub fn is_single_panel(area: Rect) -> bool {
    area.width < 100
}

/// Center a popup of given width/height in the area
pub fn centered_popup(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
