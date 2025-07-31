use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::models::Issue;
use super::app::{AppMode, EditField, GroupBy, InteractiveApp};

pub fn draw(frame: &mut Frame, app: &InteractiveApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.size());

    draw_header(frame, chunks[0], app);
    
    match app.mode {
        AppMode::Detail | AppMode::Comment | AppMode::Edit | AppMode::EditField => {
            if let Some(issue) = app.get_selected_issue() {
                draw_issue_detail(frame, chunks[1], issue);
            }
        }
        _ => draw_issues_list(frame, chunks[1], app),
    }
    
    draw_footer(frame, chunks[2], app);
    
    // Draw overlays on top of everything
    match app.mode {
        AppMode::Comment => draw_comment_overlay(frame, frame.size(), &app.comment_input),
        AppMode::Edit => draw_edit_menu_overlay(frame, frame.size(), app),
        AppMode::EditField => draw_edit_field_overlay(frame, frame.size(), app),
        _ => {}
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(30)])
        .split(area);

    let title = match app.mode {
        AppMode::Normal => " Linear Interactive Mode ",
        AppMode::Search => " Search Mode ",
        AppMode::Filter => " Filter Mode ",
        AppMode::Detail => " Issue Detail ",
        AppMode::Comment => " Add Comment ",
        AppMode::Edit => " Edit Issue ",
        AppMode::EditField => " Edit Field ",
    };

    let header = Paragraph::new(title)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, header_chunks[0]);

    let info = format!(" Issues: {} | Group by: {} ", 
        app.filtered_issues.len(),
        match app.group_by {
            GroupBy::Status => "Status",
            GroupBy::Project => "Project",
        }
    );
    let info_widget = Paragraph::new(info)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(info_widget, header_chunks[1]);
}

fn draw_issues_list(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Issues ");

    if app.loading {
        let loading = Paragraph::new("Loading issues...")
            .style(Style::default().fg(Color::Yellow))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    }

    if let Some(error) = &app.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(error_widget, area);
        return;
    }

    if app.filtered_issues.is_empty() {
        let empty = Paragraph::new("No issues found")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app.filtered_issues
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let selected = i == app.selected_index;
            let content = format!(
                "{:<10} {:<50} {:<12} {}",
                issue.identifier,
                truncate(&issue.title, 50),
                issue.state.name,
                issue.assignee.as_ref()
                    .map(|a| a.name.split_whitespace().next().unwrap_or(&a.name))
                    .unwrap_or("Unassigned")
            );
            
            let style = if selected {
                Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));
    
    frame.render_widget(list, area);

    // Draw search overlay if in search mode
    if app.mode == AppMode::Search {
        draw_search_overlay(frame, area, &app.search_query);
    }
    
    // Draw comment overlay if in comment mode
    if app.mode == AppMode::Comment {
        draw_comment_overlay(frame, area, &app.comment_input);
    }
}

fn draw_issue_detail(frame: &mut Frame, area: Rect, issue: &Issue) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // Title
            Constraint::Length(3),   // Metadata
            Constraint::Min(10),     // Description
        ])
        .split(area);

    // Title
    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" Issue ");
    let title = Paragraph::new(format!("{} - {}", issue.identifier, issue.title))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(title_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(title, chunks[0]);

    // Metadata
    let metadata = vec![
        format!("State: {} | ", issue.state.name),
        format!("Assignee: {} | ", 
            issue.assignee.as_ref()
                .map(|a| a.name.as_str())
                .unwrap_or("Unassigned")
        ),
        format!("Team: {} | ", issue.team.name),
        format!("Priority: {}", 
            match issue.priority {
                Some(0) => "None",
                Some(1) => "Low",
                Some(2) => "Medium",
                Some(3) => "High",
                Some(4) => "Urgent",
                _ => "Unknown",
            }
        ),
    ];
    let metadata_text = metadata.join("");
    let metadata_widget = Paragraph::new(metadata_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(metadata_widget, chunks[1]);

    // Description
    let description = issue.description.as_deref().unwrap_or("No description");
    let desc_widget = Paragraph::new(description)
        .style(Style::default())
        .block(Block::default().borders(Borders::ALL).title(" Description "))
        .wrap(Wrap { trim: true });
    frame.render_widget(desc_widget, chunks[2]);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let help_text = match app.mode {
        AppMode::Normal => {
            "[q] Quit  [j/k] Navigate  [Enter] View  [/] Search  [g] Toggle Group  [r] Refresh"
        }
        AppMode::Search => {
            "[Esc] Cancel  [Enter] Apply  Type to search..."
        }
        AppMode::Filter => {
            "[Esc] Back  [Enter] Apply Filter"
        }
        AppMode::Detail => {
            "[Esc/q] Back  [e] Edit  [c] Comment"
        }
        AppMode::Comment => {
            "[Esc] Cancel  [Enter] Submit  Type your comment..."
        }
        AppMode::Edit => {
            "[↑/↓] Select Field  [Enter] Edit  [Esc] Cancel"
        }
        AppMode::EditField => {
            "[Enter] Save  [Esc] Cancel  Type to edit..."
        }
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

fn draw_search_overlay(frame: &mut Frame, area: Rect, search_query: &str) {
    let popup_area = centered_rect(60, 3, area);
    
    let search_block = Block::default()
        .borders(Borders::ALL)
        .title(" Search ")
        .style(Style::default().bg(Color::Black));
    
    let search_text = Paragraph::new(format!("Search: {}_", search_query))
        .style(Style::default().fg(Color::Yellow))
        .block(search_block);
    
    frame.render_widget(search_text, popup_area);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((area.height - height) / 2),
            Constraint::Length(height),
            Constraint::Length((area.height - height) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_comment_overlay(frame: &mut Frame, area: Rect, comment_input: &str) {
    let popup_area = centered_rect(70, 10, area);
    
    // First, clear the area completely
    frame.render_widget(Clear, popup_area);
    
    // Draw a shadow/border effect around the popup
    let shadow_area = Rect {
        x: popup_area.x.saturating_sub(1),
        y: popup_area.y.saturating_sub(1),
        width: popup_area.width + 2,
        height: popup_area.height + 2,
    };
    let shadow = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(shadow, shadow_area);
    
    // Now draw the main comment box
    let comment_block = Block::default()
        .borders(Borders::ALL)
        .title("╭─ Add Comment ─╮")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Yellow).bg(Color::Black).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(comment_block.clone(), popup_area);
    
    let inner_area = comment_block.inner(popup_area);
    
    // Add some padding
    let text_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y + 1,
        width: inner_area.width.saturating_sub(2),
        height: inner_area.height.saturating_sub(2),
    };
    
    if comment_input.is_empty() {
        let help_text = vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("Type your comment below:").style(Style::default().fg(Color::Gray)),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("_").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK)),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("[Enter] Submit • [Esc] Cancel").style(Style::default().fg(Color::DarkGray)),
        ];
        let help_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center);
        frame.render_widget(help_paragraph, text_area);
    } else {
        let input_text = vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!("{}_", comment_input))
                .style(Style::default().fg(Color::White)),
        ];
        let input_paragraph = Paragraph::new(input_text)
            .wrap(Wrap { trim: true });
        frame.render_widget(input_paragraph, text_area);
        
        // Show help at bottom
        let help_area = Rect {
            x: text_area.x,
            y: text_area.y + text_area.height.saturating_sub(1),
            width: text_area.width,
            height: 1,
        };
        let help = Paragraph::new("[Enter] Submit • [Esc] Cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(help, help_area);
    }
}

fn draw_edit_menu_overlay(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let popup_area = centered_rect(60, 12, area);
    
    // Clear the area
    frame.render_widget(Clear, popup_area);
    
    // Draw shadow
    let shadow_area = Rect {
        x: popup_area.x.saturating_sub(1),
        y: popup_area.y.saturating_sub(1),
        width: popup_area.width + 2,
        height: popup_area.height + 2,
    };
    let shadow = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(shadow, shadow_area);
    
    // Draw main box
    let edit_block = Block::default()
        .borders(Borders::ALL)
        .title("╭─ Edit Issue ─╮")
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Cyan).bg(Color::Black).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(edit_block.clone(), popup_area);
    
    let inner_area = edit_block.inner(popup_area);
    
    // Create menu items
    let fields = vec![
        ("Title", 0),
        ("Description", 1),
        ("Status", 2),
        ("Assignee", 3),
        ("Priority", 4),
    ];
    
    let mut lines = vec![ratatui::text::Line::from("")];
    
    for (name, index) in fields {
        let style = if index == app.edit_field_index {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if index <= 1 {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        
        let prefix = if index == app.edit_field_index { " › " } else { "   " };
        let suffix = if index > 1 { " (not yet available)" } else { "" };
        
        lines.push(ratatui::text::Line::from(format!("{}{}{}", prefix, name, suffix)).style(style));
    }
    
    lines.push(ratatui::text::Line::from(""));
    lines.push(ratatui::text::Line::from("Use ↑/↓ to select, Enter to edit").style(Style::default().fg(Color::DarkGray)));
    
    let menu = Paragraph::new(lines);
    frame.render_widget(menu, inner_area);
}

fn draw_edit_field_overlay(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let popup_area = centered_rect(70, 10, area);
    
    // Clear the area
    frame.render_widget(Clear, popup_area);
    
    // Draw shadow
    let shadow_area = Rect {
        x: popup_area.x.saturating_sub(1),
        y: popup_area.y.saturating_sub(1),
        width: popup_area.width + 2,
        height: popup_area.height + 2,
    };
    let shadow = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default().bg(Color::Black));
    frame.render_widget(shadow, shadow_area);
    
    // Draw main box
    let field_name = match app.edit_field {
        EditField::Title => "Title",
        EditField::Description => "Description",
        EditField::Status => "Status",
        EditField::Assignee => "Assignee",
        EditField::Priority => "Priority",
    };
    
    let edit_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("╭─ Edit {} ─╮", field_name))
        .title_alignment(Alignment::Center)
        .border_style(Style::default().fg(Color::Green).bg(Color::Black).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    frame.render_widget(edit_block.clone(), popup_area);
    
    let inner_area = edit_block.inner(popup_area);
    let text_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y + 1,
        width: inner_area.width.saturating_sub(2),
        height: inner_area.height.saturating_sub(2),
    };
    
    let input_text = if app.edit_input.is_empty() {
        vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!("Current value: (empty)")).style(Style::default().fg(Color::DarkGray)),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("_").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK)),
        ]
    } else {
        vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!("{}_", app.edit_input))
                .style(Style::default().fg(Color::White)),
        ]
    };
    
    let input_paragraph = Paragraph::new(input_text)
        .wrap(Wrap { trim: true });
    frame.render_widget(input_paragraph, text_area);
    
    // Show help at bottom
    let help_area = Rect {
        x: text_area.x,
        y: text_area.y + text_area.height.saturating_sub(1),
        width: text_area.width,
        height: 1,
    };
    let help = Paragraph::new("[Enter] Save • [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, help_area);
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}