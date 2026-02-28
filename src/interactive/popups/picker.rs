use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::interactive::app::{InteractiveApp, Popup};
use crate::interactive::layout::centered_popup;
use crate::interactive::panels::list::truncate;

/// Draw the picker popup for status, priority, labels, project, or assignee.
pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let Some(popup) = &app.popup else { return };

    let (title, options, hints) = match popup {
        Popup::StatusPicker => {
            let opts: Vec<(String, Color)> = app
                .workflow_states
                .iter()
                .map(|state| {
                    let color = match state.state_type.as_str() {
                        "backlog" => Color::Gray,
                        "unstarted" => Color::LightBlue,
                        "started" => Color::Yellow,
                        "completed" => Color::Green,
                        "canceled" => Color::DarkGray,
                        _ => Color::White,
                    };
                    (state.name.clone(), color)
                })
                .collect();
            (
                "Select Status",
                opts,
                "\u{2191}/\u{2193} Navigate  Enter: Select  Esc: Cancel",
            )
        }
        Popup::PriorityPicker => {
            let opts = vec![
                ("None".to_string(), Color::Gray),
                ("Low".to_string(), Color::Blue),
                ("Medium".to_string(), Color::Yellow),
                ("High".to_string(), Color::Rgb(255, 165, 0)),
                ("Urgent".to_string(), Color::Red),
            ];
            (
                "Select Priority",
                opts,
                "\u{2191}/\u{2193} Navigate  Enter: Select  Esc: Cancel",
            )
        }
        Popup::LabelPicker => {
            let opts: Vec<(String, Color)> = app
                .available_labels
                .iter()
                .map(|label| {
                    let checkbox = if app.selected_labels.contains(&label.id) {
                        "[\u{2713}]"
                    } else {
                        "[ ]"
                    };
                    (format!("{} {}", checkbox, label.name), Color::Magenta)
                })
                .collect();
            (
                "Select Labels",
                opts,
                "Space: Toggle  Enter: Confirm  Esc: Cancel",
            )
        }
        Popup::ProjectPicker => {
            let mut opts: Vec<(String, Color)> = vec![("None".to_string(), Color::LightGreen)];
            opts.extend(
                app.available_projects
                    .iter()
                    .map(|p| (p.name.clone(), Color::LightGreen)),
            );
            (
                "Select Project",
                opts,
                "\u{2191}/\u{2193} Navigate  Enter: Select  Esc: Cancel",
            )
        }
        Popup::AssigneePicker => {
            let mut opts: Vec<(String, Color)> = vec![("Unassign".to_string(), Color::DarkGray)];
            opts.extend(
                app.team_members
                    .iter()
                    .map(|member| (member.name.clone(), Color::Cyan)),
            );
            (
                "Select Assignee",
                opts,
                "\u{2191}/\u{2193} Navigate  Enter: Select  Esc: Cancel",
            )
        }
        _ => return,
    };

    let option_count = options.len();
    let width: u16 = 40;
    let height: u16 = (option_count as u16 + 4).min(20);
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    // Determine how many option rows we can show (reserve 1 line for hints)
    let max_visible = inner.height.saturating_sub(1) as usize;

    // Calculate scroll offset to keep picker_index visible
    let scroll_offset = if app.picker_index >= max_visible {
        app.picker_index - max_visible + 1
    } else {
        0
    };

    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(max_visible)
        .map(|(i, (name, color))| {
            let display = truncate(name, (width - 4) as usize);
            let style = if i == app.picker_index {
                Style::default()
                    .fg(Color::Rgb(0, 0, 0))
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(*color)
            };
            ListItem::new(Line::from(Span::styled(format!(" {} ", display), style)))
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
        hints,
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hints_widget, hints_area);
}
