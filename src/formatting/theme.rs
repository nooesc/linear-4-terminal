#![allow(dead_code)]

use colored::{Color, Colorize};
use lazy_static::lazy_static;
use std::sync::RwLock;

/// Semantic color definitions for consistent theming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticColor {
    // Status colors
    StatusBacklog,
    StatusUnstarted,
    StatusStarted,
    StatusCompleted,
    StatusCanceled,
    
    // Priority colors
    PriorityNone,
    PriorityUrgent,
    PriorityHigh,
    PriorityMedium,
    PriorityLow,
    
    // Entity colors
    Project,
    Label,
    User,
    Assignee,
    
    // UI colors
    Header,
    Border,
    Selection,
    Highlight,
    Error,
    Warning,
    Success,
    Info,
    
    // Text colors
    Primary,
    Secondary,
    Muted,
    Link,
}

/// Theme configuration for the CLI
#[derive(Debug, Clone)]
pub struct ColorTheme {
    colors: std::collections::HashMap<SemanticColor, Color>,
}

impl ColorTheme {
    /// Create the default theme
    pub fn default() -> Self {
        let mut colors = std::collections::HashMap::new();
        
        // Status colors
        colors.insert(SemanticColor::StatusBacklog, Color::TrueColor { r: 124, g: 124, b: 124 });
        colors.insert(SemanticColor::StatusUnstarted, Color::Blue);
        colors.insert(SemanticColor::StatusStarted, Color::Yellow);
        colors.insert(SemanticColor::StatusCompleted, Color::Green);
        colors.insert(SemanticColor::StatusCanceled, Color::Red);
        
        // Priority colors
        colors.insert(SemanticColor::PriorityNone, Color::TrueColor { r: 90, g: 90, b: 90 });
        colors.insert(SemanticColor::PriorityUrgent, Color::BrightRed);
        colors.insert(SemanticColor::PriorityHigh, Color::Red);
        colors.insert(SemanticColor::PriorityMedium, Color::Yellow);
        colors.insert(SemanticColor::PriorityLow, Color::Blue);
        
        // Entity colors
        colors.insert(SemanticColor::Project, Color::Magenta);
        colors.insert(SemanticColor::Label, Color::Cyan);
        colors.insert(SemanticColor::User, Color::Green);
        colors.insert(SemanticColor::Assignee, Color::Blue);
        
        // UI colors
        colors.insert(SemanticColor::Header, Color::TrueColor { r: 21, g: 76, b: 121 });
        colors.insert(SemanticColor::Border, Color::TrueColor { r: 120, g: 120, b: 120 });
        colors.insert(SemanticColor::Selection, Color::BrightYellow);
        colors.insert(SemanticColor::Highlight, Color::BrightMagenta);
        colors.insert(SemanticColor::Error, Color::Red);
        colors.insert(SemanticColor::Warning, Color::Yellow);
        colors.insert(SemanticColor::Success, Color::Green);
        colors.insert(SemanticColor::Info, Color::Blue);
        
        // Text colors
        colors.insert(SemanticColor::Primary, Color::Black);
        colors.insert(SemanticColor::Secondary, Color::TrueColor { r: 40, g: 40, b: 40 });
        colors.insert(SemanticColor::Muted, Color::TrueColor { r: 90, g: 90, b: 90 });
        colors.insert(SemanticColor::Link, Color::Blue);
        
        Self { colors }
    }
    
    /// Get a color for a semantic meaning
    pub fn get(&self, semantic: SemanticColor) -> Color {
        self.colors.get(&semantic).copied().unwrap_or(Color::White)
    }
    
    /// Set a color for a semantic meaning
    pub fn set(&mut self, semantic: SemanticColor, color: Color) {
        self.colors.insert(semantic, color);
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::default()
    }
}

lazy_static! {
    /// Global theme instance
    static ref THEME: RwLock<ColorTheme> = RwLock::new(ColorTheme::default());
}

/// Get the current theme
pub fn current_theme() -> ColorTheme {
    THEME.read().unwrap().clone()
}

/// Set the global theme
pub fn set_theme(theme: ColorTheme) {
    *THEME.write().unwrap() = theme;
}

/// Get a color from the current theme
pub fn theme_color(semantic: SemanticColor) -> Color {
    THEME.read().unwrap().get(semantic)
}

/// Extension trait for colorizing strings with semantic colors
pub trait ThemedColorize {
    fn with_theme(&self, semantic: SemanticColor) -> colored::ColoredString;
}

impl ThemedColorize for &str {
    fn with_theme(&self, semantic: SemanticColor) -> colored::ColoredString {
        self.color(theme_color(semantic))
    }
}

impl ThemedColorize for String {
    fn with_theme(&self, semantic: SemanticColor) -> colored::ColoredString {
        self.color(theme_color(semantic))
    }
}

/// Helper functions for common color applications
pub mod helpers {
    use super::*;
    
    pub fn status_color(status_type: &str) -> SemanticColor {
        match status_type.to_lowercase().as_str() {
            "backlog" => SemanticColor::StatusBacklog,
            "unstarted" | "triage" | "todo" => SemanticColor::StatusUnstarted,
            "started" | "in_progress" | "in progress" => SemanticColor::StatusStarted,
            "completed" | "done" => SemanticColor::StatusCompleted,
            "canceled" | "cancelled" => SemanticColor::StatusCanceled,
            _ => SemanticColor::Primary,
        }
    }
    
    pub fn priority_color(priority: u8) -> SemanticColor {
        match priority {
            0 => SemanticColor::PriorityNone,
            1 => SemanticColor::PriorityLow,
            2 => SemanticColor::PriorityMedium,
            3 => SemanticColor::PriorityHigh,
            4 => SemanticColor::PriorityUrgent,
            _ => SemanticColor::Primary,
        }
    }
    
    pub fn priority_symbol(priority: u8) -> &'static str {
        match priority {
            0 => " ",
            1 => "◦",
            2 => "•",
            3 => "■",
            4 => "▲",
            _ => "?",
        }
    }
}
