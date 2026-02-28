pub mod picker;
pub mod text_input;
pub mod confirm;
pub mod create;
pub mod bulk;
pub mod help;

use ratatui::{Frame, layout::Rect};
use crate::interactive::app::{InteractiveApp, Popup};

/// Draw the active popup, if any. Draws on top of everything.
pub fn draw_popup(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let Some(popup) = &app.popup else { return };

    match popup {
        Popup::StatusPicker | Popup::PriorityPicker |
        Popup::LabelPicker | Popup::ProjectPicker |
        Popup::AssigneePicker => picker::draw(frame, area, app),
        Popup::TextInput(_) => text_input::draw(frame, area, app),
        Popup::Confirmation(_) => confirm::draw(frame, area, app),
        Popup::CreateIssue => create::draw(frame, area, app),
        Popup::BulkActions => bulk::draw(frame, area, app),
        Popup::Help => help::draw(frame, area, app),
    }
}
