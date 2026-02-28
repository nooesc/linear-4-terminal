use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use chrono::{DateTime, Utc};

use crate::interactive::app::{Focus, InteractiveApp};
use crate::models::Issue;

// ---------------------------------------------------------------------------
// Column width calculation
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ColumnWidths {
    pub id: usize,
    pub priority: usize,
    pub title: usize,
    pub project: usize,
    pub labels: usize,
    pub status: usize,
    pub assignee: usize,
    pub links: usize,
    pub age: usize,
    // Visibility flags
    pub show_project: bool,
    pub show_labels: bool,
    pub show_assignee: bool,
    pub show_links: bool,
    pub show_age: bool,
}

pub fn calculate_column_widths(available_width: u16) -> ColumnWidths {
    let width = available_width as usize;

    // Title-first approach: fixed columns take what they need, title gets the rest.
    // Columns are progressively added as width allows.
    let id_w = 7;
    let pri_w = 3; // symbol + 2 padding
    let status_w = 10;
    let age_w = 5;

    // Always-on columns cost (with 1-char separators between each)
    let fixed_cost = id_w + pri_w + status_w + age_w;

    // Optional column sizes
    let project_w = 10;
    let labels_w = 12;
    let assignee_w = 10;
    let links_w = 3;

    // Determine which optional columns fit. We need at least 15 chars for title.
    let mut used = fixed_cost;
    let show_project = width.saturating_sub(used + project_w) >= 15;
    if show_project { used += project_w; }

    let show_labels = width >= 120 && width.saturating_sub(used + labels_w) >= 20;
    if show_labels { used += labels_w; }

    let show_assignee = width >= 140 && width.saturating_sub(used + assignee_w) >= 25;
    if show_assignee { used += assignee_w; }

    let show_links = width >= 170 && width.saturating_sub(used + links_w) >= 30;
    if show_links { used += links_w; }

    // Title gets ALL remaining space — no cap
    let title_w = width.saturating_sub(used).max(10);

    ColumnWidths {
        id: id_w,
        priority: pri_w,
        title: title_w,
        project: if show_project { project_w } else { 0 },
        labels: if show_labels { labels_w } else { 0 },
        status: status_w,
        assignee: if show_assignee { assignee_w } else { 0 },
        links: if show_links { links_w } else { 0 },
        age: age_w,
        show_project,
        show_labels,
        show_assignee,
        show_links,
        show_age: true,
    }
}

// ---------------------------------------------------------------------------
// Helper functions (public for reuse by other modules)
// ---------------------------------------------------------------------------

pub fn truncate(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        return s.chars().take(max_width).collect();
    }
    if s.len() <= max_width {
        s.to_string()
    } else {
        format!("{}...", &s[..max_width.saturating_sub(3)])
    }
}

pub fn truncate_id(id: &str, max_width: usize) -> String {
    if id.len() <= max_width {
        id.to_string()
    } else {
        // Try to extract just the number part for very narrow displays
        if let Some(dash_pos) = id.find('-') {
            let number_part = &id[dash_pos + 1..];
            if number_part.len() <= max_width {
                return number_part.to_string();
            }
        }
        truncate(id, max_width)
    }
}

pub fn format_age(created_at: &str) -> String {
    if let Ok(created) = DateTime::parse_from_rfc3339(created_at) {
        let now = Utc::now();
        let duration = now.signed_duration_since(created.with_timezone(&Utc));

        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days >= 7 {
            let weeks = days / 7;
            let remaining_days = days % 7;
            if remaining_days > 0 {
                format!("{}w{}d", weeks, remaining_days)
            } else {
                format!("{}w", weeks)
            }
        } else if days > 0 {
            if hours > 0 {
                format!("{}d{}h", days, hours)
            } else {
                format!("{}d", days)
            }
        } else if hours > 0 {
            if minutes > 0 {
                format!("{}h{}m", hours, minutes)
            } else {
                format!("{}h", hours)
            }
        } else if minutes > 0 {
            format!("{}m", minutes)
        } else {
            "< 1m".to_string()
        }
    } else {
        "-".to_string()
    }
}

pub fn parse_assignee_name(user: &crate::models::User) -> String {
    // First try to extract username from email
    if let Some(username) = user.email.split('@').next() {
        if !username.is_empty() {
            return username.to_string();
        }
    }

    // Otherwise, try to get first name
    if let Some(first_name) = user.name.split_whitespace().next() {
        if !first_name.is_empty() {
            return first_name.to_string();
        }
    }

    // Fallback to full name
    user.name.clone()
}

fn extract_links_from_text(text: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Match URLs (http/https)
    if let Ok(url_regex) = regex::Regex::new(r#"https?://[^\s<>"{}|\\^`\[\]]+"#) {
        for capture in url_regex.captures_iter(text) {
            links.push(capture[0].to_string());
        }
    }

    // Match markdown links [text](url)
    if let Ok(md_link_regex) = regex::Regex::new(r#"\[([^\]]+)\]\(([^)]+)\)"#) {
        for capture in md_link_regex.captures_iter(text) {
            if let Some(url) = capture.get(2) {
                links.push(url.as_str().to_string());
            }
        }
    }

    links
}

pub fn get_issue_links(issue: &Issue) -> Vec<String> {
    let mut all_links = vec![issue.url.clone()]; // Always include the Linear URL

    if let Some(desc) = &issue.description {
        all_links.extend(extract_links_from_text(desc));
    }

    // Deduplicate
    all_links.sort();
    all_links.dedup();
    all_links
}

// ---------------------------------------------------------------------------
// Priority / Status helpers
// ---------------------------------------------------------------------------

fn priority_symbol_and_color(priority: Option<u8>) -> (&'static str, Color) {
    match priority {
        Some(0) => (" ", Color::Gray),
        Some(1) => ("\u{25e6}", Color::Blue),      // ◦
        Some(2) => ("\u{2022}", Color::Yellow),     // •
        Some(3) => ("\u{25a0}", Color::Rgb(255, 165, 0)), // ■  Orange
        Some(4) => ("\u{25b2}", Color::Red),        // ▲
        _ => (" ", Color::Gray),
    }
}

pub fn status_color(state_type: &str) -> Color {
    match state_type {
        "backlog" => Color::Gray,
        "unstarted" => Color::LightBlue,
        "started" => Color::Yellow,
        "completed" => Color::Green,
        "canceled" => Color::DarkGray,
        _ => Color::White,
    }
}

// ---------------------------------------------------------------------------
// Main draw function
// ---------------------------------------------------------------------------

pub fn draw_list(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::IssueList;
    let border_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Issues ")
        .border_style(border_style);

    // Loading state
    if app.loading {
        let loading = Paragraph::new("Loading issues...")
            .style(Style::default().fg(Color::Yellow))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    }

    // Error state
    if let Some(error) = &app.error_message {
        let error_widget = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(block);
        frame.render_widget(error_widget, area);
        return;
    }

    // Empty state
    if app.filtered_issues.is_empty() {
        let empty = Paragraph::new("No issues found")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    // Calculate column widths based on available space
    let inner_width = area.width.saturating_sub(2); // Account for borders
    let col_widths = calculate_column_widths(inner_width);

    // Build dynamic header row
    let header_style = Style::default()
        .fg(Color::Gray)
        .add_modifier(Modifier::UNDERLINED);
    let mut header = format!(
        "{:<id_w$}{:<pri_w$}{:<title_w$}",
        "ID", "P", "Title",
        id_w = col_widths.id,
        pri_w = col_widths.priority,
        title_w = col_widths.title,
    );

    if col_widths.show_project {
        header.push_str(&format!("{:<w$}", "Project", w = col_widths.project));
    }
    if col_widths.show_labels {
        header.push_str(&format!("{:<w$}", "Labels", w = col_widths.labels));
    }
    header.push_str(&format!("{:<w$}", "Status", w = col_widths.status));
    if col_widths.show_assignee {
        header.push_str(&format!("{:<w$}", "Assignee", w = col_widths.assignee));
    }
    if col_widths.show_links {
        header.push_str(&format!("{:<w$}", "\u{1f517}", w = col_widths.links));
    }
    if col_widths.show_age {
        header.push_str(&format!("{:<w$}", "Age", w = col_widths.age));
    }

    let header_item = ListItem::new(header).style(header_style);

    // Viewport scrolling: 1 row for header, rest for issues
    let inner_height = area.height.saturating_sub(2) as usize; // -2 for borders
    let visible_issues = inner_height.saturating_sub(1); // -1 for header row
    let scroll_offset = if visible_issues > 0 && app.selected_index >= visible_issues {
        app.selected_index - visible_issues + 1
    } else {
        0
    };

    // Build visible issue rows
    let items: Vec<ListItem> = std::iter::once(header_item)
        .chain(
            app.filtered_issues
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(visible_issues)
                .map(|(i, issue)| build_row(i, issue, app, &col_widths)),
        )
        .collect();

    // Build title with scroll position and active filter
    let total = app.filtered_issues.len();
    let mut title = if scroll_offset > 0 || total > visible_issues {
        format!(
            " Issues [{}-{}/{}]",
            scroll_offset + 1,
            (scroll_offset + visible_issues).min(total),
            total
        )
    } else {
        format!(" Issues ({})", total)
    };
    if !app.filter_query.is_empty() {
        title.push_str(&format!(" [filter: {}]", app.filter_query));
    }
    if !app.search_query.is_empty() {
        title.push_str(&format!(" [search: {}]", app.search_query));
    }
    title.push(' ');
    let block = block.title(title);

    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

// ---------------------------------------------------------------------------
// Row builder
// ---------------------------------------------------------------------------

fn build_row<'a>(
    index: usize,
    issue: &Issue,
    app: &InteractiveApp,
    col_widths: &ColumnWidths,
) -> ListItem<'a> {
    let selected = index == app.selected_index;
    let multi = app.multi_selected.contains(&index);

    // Row background
    let row_bg = if selected {
        Some(Color::Rgb(30, 35, 50))
    } else {
        None
    };

    let (priority_symbol, priority_color) = priority_symbol_and_color(issue.priority);
    let st_color = status_color(&issue.state.state_type);

    let assignee_name = issue
        .assignee
        .as_ref()
        .map(|a| parse_assignee_name(a))
        .unwrap_or_else(|| "Unassigned".to_string());

    // ID column — prepend checkmark for multi-selected rows
    let id_text = if multi {
        let raw_id = truncate_id(&issue.identifier, col_widths.id.saturating_sub(2));
        format!(
            "\u{2713} {:<width$}",
            raw_id,
            width = col_widths.id.saturating_sub(2)
        )
    } else {
        format!(
            "{:<width$}",
            truncate_id(&issue.identifier, col_widths.id),
            width = col_widths.id
        )
    };

    let id_span = Span::styled(id_text, Style::default());

    let priority_span = Span::styled(
        format!("{} ", priority_symbol),
        Style::default().fg(priority_color),
    );

    let title_span = Span::styled(
        format!(
            "{:<width$}",
            truncate(&issue.title, col_widths.title),
            width = col_widths.title
        ),
        Style::default(),
    );

    let status_style = if selected {
        Style::default()
            .fg(st_color)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(st_color)
    };
    let status_span = Span::styled(
        format!(
            "{:<width$}",
            truncate(&issue.state.name, col_widths.status),
            width = col_widths.status
        ),
        status_style,
    );

    // Build dynamic row spans
    let mut spans = vec![id_span, priority_span, title_span];

    // Project column
    if col_widths.show_project {
        let project_name = issue
            .project
            .as_ref()
            .map(|p| p.name.as_str())
            .unwrap_or("-");
        let project_span = Span::styled(
            format!(
                "{:<width$}",
                truncate(project_name, col_widths.project),
                width = col_widths.project
            ),
            Style::default().fg(Color::LightGreen),
        );
        spans.push(project_span);
    }

    // Labels column
    if col_widths.show_labels {
        let labels_text = if issue.labels.nodes.is_empty() {
            "-".to_string()
        } else {
            let labels: Vec<&str> = issue
                .labels
                .nodes
                .iter()
                .take(2)
                .map(|l| l.name.as_str())
                .collect();
            labels.join(", ")
        };
        let labels_span = Span::styled(
            format!(
                "{:<width$}",
                truncate(&labels_text, col_widths.labels),
                width = col_widths.labels
            ),
            Style::default().fg(Color::Magenta),
        );
        spans.push(labels_span);
    }

    spans.push(status_span);

    // Assignee column
    if col_widths.show_assignee {
        let assignee_span = Span::styled(
            format!(
                "{:<width$}",
                truncate(&assignee_name, col_widths.assignee),
                width = col_widths.assignee
            ),
            Style::default().fg(Color::Cyan),
        );
        spans.push(assignee_span);
    }

    // Links column
    if col_widths.show_links {
        let links = get_issue_links(issue);
        let extra_links_count = if links.len() > 1 { links.len() - 1 } else { 0 };
        let links_text = if extra_links_count > 0 {
            format!("{} ", extra_links_count)
        } else {
            "  ".to_string()
        };
        let links_span = Span::styled(links_text, Style::default().fg(Color::Blue));
        spans.push(links_span);
    }

    // Age column
    if col_widths.show_age {
        let age_text = format_age(&issue.created_at);
        let age_span = Span::styled(
            format!("{:<width$}", age_text, width = col_widths.age),
            Style::default().fg(Color::Gray),
        );
        spans.push(age_span);
    }

    let line = Line::from(spans);
    let item = ListItem::new(line);
    if let Some(bg) = row_bg {
        item.style(Style::default().bg(bg))
    } else {
        item
    }
}
