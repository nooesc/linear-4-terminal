use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::models::Issue;
use super::app::{AppMode, GroupBy, InteractiveApp};

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
        AppMode::Detail => {
            if let Some(issue) = app.get_selected_issue() {
                draw_issue_detail(frame, chunks[1], issue);
            }
        }
        _ => draw_issues_list(frame, chunks[1], app),
    }
    
    draw_footer(frame, chunks[2], app);
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

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        s.to_string()
    } else {
        format!("{}...", &s[..max_width - 3])
    }
}