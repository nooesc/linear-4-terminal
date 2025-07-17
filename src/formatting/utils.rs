use colored::*;
use chrono::{DateTime, Utc};

pub fn extract_first_name(name: &str) -> &str {
    name.split_whitespace()
        .next()
        .unwrap_or(name)
}

pub fn format_priority(priority: Option<u8>) -> ColoredString {
    match priority {
        Some(4) => "Urgent".red().bold(),
        Some(3) => "High".red(),
        Some(2) => "Medium".yellow(),
        Some(1) => "Low".normal(),
        _ => "None".dimmed(),
    }
}

pub fn format_priority_indicator(priority: Option<u8>) -> ColoredString {
    match priority {
        Some(4) => "!".red().bold(),
        Some(3) => "!".red(),
        Some(2) => "!".yellow(),
        _ => " ".normal(),
    }
}

pub fn format_relative_time(timestamp: &str) -> String {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(timestamp) {
        let now = Utc::now();
        let duration = now.signed_duration_since(parsed);
        
        if duration.num_days() > 365 {
            format!("{}y ago", duration.num_days() / 365)
        } else if duration.num_days() > 30 {
            format!("{}mo ago", duration.num_days() / 30)
        } else if duration.num_days() > 0 {
            format!("{}d ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{}h ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{}m ago", duration.num_minutes())
        } else {
            "just now".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn clean_description(desc: &str) -> String {
    // Take first non-empty line
    let first_line = desc
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    
    // Remove markdown formatting for display
    let cleaned = first_line
        .trim()
        .replace("**", "")
        .replace("*", "")
        .replace("_", "")
        .replace("`", "")
        .replace("#", "")
        .replace(">", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "");
    
    // Ensure it ends with proper punctuation
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    
    if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        trimmed.to_string()
    } else {
        format!("{}.", trimmed)
    }
}