use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::interactive::app::{Focus, InteractiveApp};
use crate::models::Issue;

use super::list::{format_age, parse_assignee_name, status_color, truncate};

// ---------------------------------------------------------------------------
// Public draw entry point
// ---------------------------------------------------------------------------

pub fn draw_detail(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::DetailPanel;
    let border_style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let issue = match app.get_selected_issue() {
        Some(i) => i,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Detail ")
                .border_style(border_style);
            let empty = Paragraph::new("No issue selected")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty.block(block), area);
            return;
        }
    };

    // Split into three sections: info, description, comments
    let comments_height = 10u16;
    let info_height = 8u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(info_height),
            Constraint::Min(6),
            Constraint::Length(comments_height),
        ])
        .split(area);

    draw_info_section(frame, chunks[0], issue, border_style);
    draw_description_section(frame, chunks[1], issue, app.detail_scroll, border_style);
    draw_comments_section(frame, chunks[2], app, border_style);
}

// ---------------------------------------------------------------------------
// Info section
// ---------------------------------------------------------------------------

fn draw_info_section(frame: &mut Frame, area: Rect, issue: &Issue, border_style: Style) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Info ")
        .border_style(border_style);

    let st_color = status_color(&issue.state.state_type);

    let (priority_name, priority_color) = match issue.priority {
        Some(0) => ("None", Color::Gray),
        Some(1) => ("Low", Color::Blue),
        Some(2) => ("Medium", Color::Yellow),
        Some(3) => ("High", Color::Rgb(255, 165, 0)),
        Some(4) => ("Urgent", Color::Red),
        _ => ("Unknown", Color::Gray),
    };

    let assignee_text = issue
        .assignee
        .as_ref()
        .map(|a| parse_assignee_name(a))
        .unwrap_or_else(|| "Unassigned".to_string());

    let project_text = issue
        .project
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("None");

    let labels_text = if issue.labels.nodes.is_empty() {
        "None".to_string()
    } else {
        issue
            .labels
            .nodes
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Title line
    let title_line = Line::from(vec![Span::styled(
        format!("{} - {}", issue.identifier, issue.title),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    // Status + Priority
    let status_priority_line = Line::from(vec![
        Span::raw("Status: "),
        Span::styled(
            &issue.state.name,
            Style::default()
                .fg(st_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Priority: "),
        Span::styled(
            priority_name,
            Style::default()
                .fg(priority_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    // Assignee + Project
    let assignee_project_line = Line::from(vec![
        Span::raw("Assignee: "),
        Span::styled(&assignee_text, Style::default().fg(Color::Cyan)),
        Span::raw("  Project: "),
        Span::styled(project_text, Style::default().fg(Color::LightGreen)),
    ]);

    // Labels
    let labels_line = Line::from(vec![
        Span::raw("Labels: "),
        Span::styled(labels_text, Style::default().fg(Color::Magenta)),
    ]);

    let info = Paragraph::new(vec![
        title_line,
        Line::from(""),
        status_priority_line,
        assignee_project_line,
        labels_line,
    ])
    .block(block)
    .wrap(Wrap { trim: true });

    frame.render_widget(info, area);
}

// ---------------------------------------------------------------------------
// Description section (with markdown rendering)
// ---------------------------------------------------------------------------

fn draw_description_section(
    frame: &mut Frame,
    area: Rect,
    issue: &Issue,
    scroll: u16,
    border_style: Style,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Description ")
        .border_style(border_style);

    match &issue.description {
        Some(desc) if !desc.trim().is_empty() => {
            let lines = render_markdown_to_lines(desc);
            let desc_widget = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: true })
                .scroll((scroll, 0));
            frame.render_widget(desc_widget, area);
        }
        _ => {
            let empty = Paragraph::new("No description")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            frame.render_widget(empty, area);
        }
    }
}

// ---------------------------------------------------------------------------
// Comments section
// ---------------------------------------------------------------------------

fn draw_comments_section(frame: &mut Frame, area: Rect, app: &InteractiveApp, border_style: Style) {
    let comment_count = app.comments.len();
    let title = format!(" Comments ({}) ", comment_count);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if app.comments_loading {
        let loading = Paragraph::new("Loading comments...")
            .style(Style::default().fg(Color::Yellow))
            .block(block);
        frame.render_widget(loading, area);
        return;
    }

    if app.comments.is_empty() {
        let empty = Paragraph::new("No comments")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();
    for comment in &app.comments {
        let author = comment
            .user
            .as_ref()
            .map(|u| parse_assignee_name(u))
            .unwrap_or_else(|| "Unknown".to_string());
        let age = format_age(&comment.created_at);

        // First line: author (time)
        let header_line = Line::from(vec![
            Span::styled(
                author,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" ({})", age), Style::default().fg(Color::Gray)),
            Span::raw(": "),
        ]);
        lines.push(header_line);

        // Body — take first line only to keep compact
        let body_first_line = comment.body.lines().next().unwrap_or("");
        let body_text = truncate(body_first_line, area.width.saturating_sub(4) as usize);
        lines.push(Line::from(Span::raw(body_text)));
        lines.push(Line::from(""));
    }

    let comments_widget = Paragraph::new(lines).block(block);
    frame.render_widget(comments_widget, area);
}

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

fn render_markdown_to_lines(text: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let text_lines: Vec<&str> = text.lines().collect();
    let mut in_code_block = false;

    for (i, line) in text_lines.iter().enumerate() {
        let trimmed = line.trim();

        // Handle code block delimiters
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                lines.push(Line::from(vec![Span::styled(
                    "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}".to_string(),
                    Style::default().fg(Color::DarkGray),
                )]));
            } else {
                lines.push(Line::from(vec![Span::styled(
                    "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}".to_string(),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(vec![
                Span::styled(
                    "\u{2502} ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(line.to_string(), Style::default().fg(Color::Cyan)),
            ]));
            continue;
        }

        // Headers
        if trimmed.starts_with("### ") {
            let header = trimmed.trim_start_matches("### ");
            lines.push(Line::from(vec![]));
            lines.push(Line::from(vec![Span::styled(
                header.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )]));
            continue;
        } else if trimmed.starts_with("## ") {
            let header = trimmed.trim_start_matches("## ");
            lines.push(Line::from(vec![]));
            lines.push(Line::from(vec![Span::styled(
                header.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "\u{2500}".repeat(header.len()),
                Style::default().fg(Color::DarkGray),
            )]));
            continue;
        } else if trimmed.starts_with("# ") {
            let header = trimmed.trim_start_matches("# ");
            lines.push(Line::from(vec![]));
            lines.push(Line::from(vec![Span::styled(
                header.to_string(),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )]));
            lines.push(Line::from(vec![Span::styled(
                "\u{2550}".repeat(header.len()),
                Style::default().fg(Color::DarkGray),
            )]));
            continue;
        }

        // Unordered lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let content = trimmed[2..].trim();
            let formatted = render_inline_markdown(content);
            let mut list_line =
                vec![Span::styled("  \u{2022} ".to_string(), Style::default().fg(Color::Yellow))];
            list_line.extend(formatted);
            lines.push(Line::from(list_line));
            continue;
        }

        // Numbered lists
        if let Some(captures) = regex::Regex::new(r"^(\d+)\.\s+(.*)$")
            .ok()
            .and_then(|re| re.captures(trimmed))
        {
            let number = captures.get(1).map(|m| m.as_str()).unwrap_or("1");
            let content = captures.get(2).map(|m| m.as_str()).unwrap_or("");
            let formatted = render_inline_markdown(content);
            let mut list_line = vec![
                Span::raw("  ".to_string()),
                Span::styled(number.to_string(), Style::default().fg(Color::Cyan)),
                Span::raw(". ".to_string()),
            ];
            list_line.extend(formatted);
            lines.push(Line::from(list_line));
            continue;
        }

        // Blockquotes
        if trimmed.starts_with("> ") {
            let content = trimmed[2..].trim();
            let formatted = render_inline_markdown(content);
            let mut quote_line = vec![Span::styled(
                "\u{2502} ".to_string(),
                Style::default().fg(Color::DarkGray),
            )];
            quote_line.extend(formatted);
            lines.push(Line::from(quote_line));
            continue;
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            lines.push(Line::from(vec![Span::styled(
                "\u{2500}".repeat(40),
                Style::default().fg(Color::DarkGray),
            )]));
            continue;
        }

        // Regular paragraphs
        if !trimmed.is_empty() {
            lines.push(Line::from(render_inline_markdown(line)));
        } else if i > 0 && i < text_lines.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines
}

// ---------------------------------------------------------------------------
// Inline markdown rendering
// ---------------------------------------------------------------------------

fn render_inline_markdown(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut remaining = text.to_string();

    while !remaining.is_empty() {
        // Check for inline code
        if let Some(code_start) = remaining.find('`') {
            if let Some(code_end) = remaining[code_start + 1..].find('`') {
                // Text before code
                if code_start > 0 {
                    spans.extend(process_text_formatting(&remaining[..code_start]));
                }
                // The code span
                let code_text = &remaining[code_start + 1..code_start + 1 + code_end];
                spans.push(Span::styled(
                    code_text.to_string(),
                    Style::default().bg(Color::Rgb(40, 40, 50)).fg(Color::White),
                ));
                remaining = remaining[code_start + code_end + 2..].to_string();
                continue;
            }
        }

        // No more special elements, process the rest
        spans.extend(process_text_formatting(&remaining));
        break;
    }

    spans
}

fn process_text_formatting(text: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut current_text = String::new();

    'outer: while i < chars.len() {
        // Bold: **text** or __text__
        if i + 1 < chars.len()
            && ((chars[i] == '*' && chars[i + 1] == '*')
                || (chars[i] == '_' && chars[i + 1] == '_'))
        {
            let delimiter = chars[i];
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == delimiter && chars[j + 1] == delimiter {
                    if !current_text.is_empty() {
                        spans.push(Span::raw(current_text.clone()));
                        current_text.clear();
                    }
                    if j > i + 2 {
                        let bold_text: String = chars[i + 2..j].iter().collect();
                        spans.push(Span::styled(
                            bold_text,
                            Style::default().add_modifier(Modifier::BOLD),
                        ));
                    }
                    i = j + 2;
                    continue 'outer;
                }
                j += 1;
            }
        }

        // Italic: *text* or _text_
        if chars[i] == '*' || chars[i] == '_' {
            let delimiter = chars[i];
            let is_bold = i + 1 < chars.len() && chars[i + 1] == delimiter;
            if !is_bold {
                let mut j = i + 1;
                while j < chars.len() {
                    if chars[j] == delimiter {
                        if !current_text.is_empty() {
                            spans.push(Span::raw(current_text.clone()));
                            current_text.clear();
                        }
                        if j > i + 1 {
                            let italic_text: String = chars[i + 1..j].iter().collect();
                            spans.push(Span::styled(
                                italic_text,
                                Style::default().add_modifier(Modifier::ITALIC),
                            ));
                        }
                        i = j + 1;
                        continue 'outer;
                    }
                    j += 1;
                }
            }
        }

        // Links: [text](url)
        if chars[i] == '[' {
            let mut j = i + 1;
            while j < chars.len() && chars[j] != ']' {
                j += 1;
            }
            if j < chars.len() && j + 1 < chars.len() && chars[j + 1] == '(' {
                let mut k = j + 2;
                while k < chars.len() && chars[k] != ')' {
                    k += 1;
                }
                if k < chars.len() {
                    if !current_text.is_empty() {
                        spans.push(Span::raw(current_text.clone()));
                        current_text.clear();
                    }
                    if j > i + 1 {
                        let link_text: String = chars[i + 1..j].iter().collect();
                        spans.push(Span::styled(
                            link_text,
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::UNDERLINED),
                        ));
                    }
                    i = k + 1;
                    continue 'outer;
                }
            }
        }

        // Regular character
        current_text.push(chars[i]);
        i += 1;
    }

    if !current_text.is_empty() {
        spans.push(Span::raw(current_text));
    }

    spans
}
