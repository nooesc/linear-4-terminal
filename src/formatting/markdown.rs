use colored::*;
use regex::Regex;

pub fn format_links(text: &str) -> String {
    let link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    let mut result = text.to_string();
    
    for cap in link_regex.captures_iter(text) {
        let link_text = &cap[1];
        let link_url = &cap[2];
        let formatted_link = format!("{} ({})", link_text.blue().underline(), link_url.dimmed());
        result = result.replace(&cap[0], &formatted_link);
    }
    
    result
}

pub fn format_bold(text: &str) -> String {
    let bold_regex = Regex::new(r"\*\*([^*]+)\*\*").unwrap();
    let mut result = text.to_string();
    
    for cap in bold_regex.captures_iter(text) {
        let bold_text = &cap[1];
        let formatted_bold = bold_text.bold().to_string();
        result = result.replace(&cap[0], &formatted_bold);
    }
    
    // Also handle single asterisks for bold (some markdown uses this)
    let single_bold_regex = Regex::new(r"\*([^*]+)\*").unwrap();
    for cap in single_bold_regex.captures_iter(&result.clone()) {
        let bold_text = &cap[1];
        let formatted_bold = bold_text.bold().to_string();
        result = result.replace(&cap[0], &formatted_bold);
    }
    
    result
}

pub fn format_italic(text: &str) -> String {
    let italic_regex = Regex::new(r"_([^_]+)_").unwrap();
    let mut result = text.to_string();
    
    for cap in italic_regex.captures_iter(text) {
        let italic_text = &cap[1];
        let formatted_italic = italic_text.italic().to_string();
        result = result.replace(&cap[0], &formatted_italic);
    }
    
    // Also handle markdown *text* for italics (when not bold)
    let md_italic_regex = Regex::new(r"(?<!\*)\*(?!\*)([^*]+)\*(?!\*)").unwrap();
    for cap in md_italic_regex.captures_iter(&result.clone()) {
        let italic_text = &cap[1];
        let formatted_italic = italic_text.italic().to_string();
        result = result.replace(&cap[0], &formatted_italic);
    }
    
    result
}

pub fn format_markdown(text: &str) -> String {
    let mut formatted = String::new();
    let lines: Vec<&str> = text.lines().collect();
    let mut in_code_block = false;
    let _list_stack: Vec<&str> = Vec::new();
    
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Handle code blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                formatted.push_str(&format!("\n{}\n", "─".repeat(40).dimmed()));
            } else {
                formatted.push_str(&format!("{}\n", "─".repeat(40).dimmed()));
            }
            continue;
        }
        
        if in_code_block {
            formatted.push_str(&format!("{}\n", line.dimmed()));
            continue;
        }
        
        // Handle headers
        if trimmed.starts_with("# ") {
            let header = trimmed.trim_start_matches("# ");
            formatted.push_str(&format!("\n{}\n{}\n", header.bold().blue(), "═".repeat(header.len()).blue()));
            continue;
        } else if trimmed.starts_with("## ") {
            let header = trimmed.trim_start_matches("## ");
            formatted.push_str(&format!("\n{}\n{}\n", header.bold().cyan(), "─".repeat(header.len()).cyan()));
            continue;
        } else if trimmed.starts_with("### ") {
            let header = trimmed.trim_start_matches("### ");
            formatted.push_str(&format!("\n{}\n", header.bold().green()));
            continue;
        }
        
        // Handle lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let list_content = trimmed[2..].trim();
            let indent_level = line.len() - line.trim_start().len();
            let indent = " ".repeat(indent_level);
            let formatted_content = format_inline_markdown(list_content);
            formatted.push_str(&format!("{}• {}\n", indent, formatted_content));
            continue;
        }
        
        // Handle numbered lists
        if let Some(cap) = Regex::new(r"^(\d+)\.\s+(.*)$").unwrap().captures(trimmed) {
            let number = &cap[1];
            let list_content = &cap[2];
            let indent_level = line.len() - line.trim_start().len();
            let indent = " ".repeat(indent_level);
            let formatted_content = format_inline_markdown(list_content);
            formatted.push_str(&format!("{}{}. {}\n", indent, number.cyan(), formatted_content));
            continue;
        }
        
        // Handle blockquotes
        if trimmed.starts_with("> ") {
            let quote_content = trimmed[2..].trim();
            let formatted_content = format_inline_markdown(quote_content);
            formatted.push_str(&format!("│ {}\n", formatted_content.dimmed()));
            continue;
        }
        
        // Handle horizontal rules
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            formatted.push_str(&format!("{}\n", "─".repeat(40).dimmed()));
            continue;
        }
        
        // Handle inline code
        let code_regex = Regex::new(r"`([^`]+)`").unwrap();
        let mut line_formatted = line.to_string();
        for cap in code_regex.captures_iter(line) {
            let code_text = &cap[1];
            let formatted_code = code_text.on_black().white().to_string();
            line_formatted = line_formatted.replace(&cap[0], &formatted_code);
        }
        
        // Apply inline formatting
        line_formatted = format_inline_markdown(&line_formatted);
        
        // Handle empty lines
        if trimmed.is_empty() {
            // Only add empty line if not between list items
            if i > 0 && i < lines.len() - 1 {
                let prev_line = lines[i - 1].trim();
                let next_line = lines[i + 1].trim();
                let prev_is_list = prev_line.starts_with("- ") || prev_line.starts_with("* ") || 
                                  Regex::new(r"^\d+\.\s").unwrap().is_match(prev_line);
                let next_is_list = next_line.starts_with("- ") || next_line.starts_with("* ") || 
                                  Regex::new(r"^\d+\.\s").unwrap().is_match(next_line);
                
                if !(prev_is_list && next_is_list) {
                    formatted.push('\n');
                }
            } else {
                formatted.push('\n');
            }
        } else {
            formatted.push_str(&line_formatted);
            formatted.push('\n');
        }
    }
    
    // Remove trailing newline
    if formatted.ends_with('\n') {
        formatted.pop();
    }
    
    formatted
}

pub fn print_formatted_markdown(text: &str) {
    println!("{}", format_markdown(text));
}

pub fn format_inline_markdown(text: &str) -> String {
    let mut result = text.to_string();
    
    // Format links
    result = format_links(&result);
    
    // Format bold (must come before italic to handle ** correctly)
    result = format_bold(&result);
    
    // Format italic
    result = format_italic(&result);
    
    // Format inline code
    let code_regex = Regex::new(r"`([^`]+)`").unwrap();
    for cap in code_regex.captures_iter(&result.clone()) {
        let code_text = &cap[1];
        let formatted_code = code_text.on_black().white().to_string();
        result = result.replace(&cap[0], &formatted_code);
    }
    
    result
}