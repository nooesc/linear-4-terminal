use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::interactive::app::InteractiveApp;
use crate::interactive::layout::centered_popup;
use crate::interactive::panels::list::truncate;

/// Draw the issue creation form popup.
pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let width: u16 = 60;
    let height: u16 = 14;
    let popup_area = centered_popup(width, height, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" New Issue ")
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let form = &app.create_form;
    let max_value_width = (inner.width as usize).saturating_sub(14);

    let fields: Vec<(&str, String)> = vec![
        ("Title", {
            if form.title.is_empty() {
                "<enter title>".to_string()
            } else {
                truncate(&form.title, max_value_width)
            }
        }),
        ("Team", {
            form.team_id
                .as_ref()
                .and_then(|tid| {
                    // Look up the team name — we only have issues loaded,
                    // so try to find a matching team from issues.
                    app.issues.iter().find_map(|i| {
                        if i.team.id == *tid {
                            Some(i.team.name.clone())
                        } else {
                            None
                        }
                    })
                })
                .unwrap_or_else(|| "Select...".to_string())
        }),
        ("Status", {
            form.status_id
                .as_ref()
                .and_then(|sid| {
                    app.workflow_states
                        .iter()
                        .find(|s| s.id == *sid)
                        .map(|s| s.name.clone())
                })
                .unwrap_or_else(|| "Backlog".to_string())
        }),
        ("Priority", {
            match form.priority {
                Some(0) | None => "None".to_string(),
                Some(1) => "Low".to_string(),
                Some(2) => "Medium".to_string(),
                Some(3) => "High".to_string(),
                Some(4) => "Urgent".to_string(),
                Some(n) => format!("P{}", n),
            }
        }),
        ("Project", {
            form.project_id
                .as_ref()
                .and_then(|pid| {
                    app.available_projects
                        .iter()
                        .find(|p| p.id == *pid)
                        .map(|p| p.name.clone())
                })
                .unwrap_or_else(|| "None".to_string())
        }),
        ("Labels", {
            if form.label_ids.is_empty() {
                "None".to_string()
            } else {
                let count = form.label_ids.len();
                let names: Vec<String> = form
                    .label_ids
                    .iter()
                    .filter_map(|lid| {
                        app.available_labels
                            .iter()
                            .find(|l| l.id == *lid)
                            .map(|l| l.name.clone())
                    })
                    .collect();
                if names.is_empty() {
                    format!("{} selected", count)
                } else {
                    truncate(&names.join(", "), max_value_width)
                }
            }
        }),
        ("Assignee", {
            form.assignee_id
                .as_ref()
                .and_then(|aid| {
                    app.team_members
                        .iter()
                        .find(|m| m.id == *aid)
                        .map(|m| m.name.clone())
                })
                .unwrap_or_else(|| "None".to_string())
        }),
    ];

    for (i, (label, value)) in fields.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height.saturating_sub(1) {
            break;
        }

        let is_active = i == form.active_field;

        let label_style = if is_active {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if is_active {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let indicator = if is_active { "\u{25b6} " } else { "  " };

        let line = Line::from(vec![
            Span::styled(indicator, label_style),
            Span::styled(format!("{:<10}", label), label_style),
            Span::styled(value.clone(), value_style),
        ]);

        let row_area = Rect::new(inner.x, y, inner.width, 1);
        frame.render_widget(Paragraph::new(line), row_area);
    }

    // Hints at the bottom
    let hints_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    let hints_widget = Paragraph::new(Line::from(Span::styled(
        "Tab: Next field  Enter: Edit/Create  Esc: Cancel",
        Style::default().fg(Color::DarkGray),
    )));
    frame.render_widget(hints_widget, hints_area);
}
