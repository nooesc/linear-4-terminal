use crate::formatting::theme::{SemanticColor, ColorTheme, theme_color, ThemedColorize};
use crate::formatting::theme::helpers::{status_color, priority_color, priority_symbol};
use colored::Color;

#[test]
fn test_default_theme() {
    let theme = ColorTheme::default();
    
    // Test status colors
    assert_eq!(theme.get(SemanticColor::StatusCompleted), Color::Green);
    assert_eq!(theme.get(SemanticColor::StatusStarted), Color::Yellow);
    assert_eq!(theme.get(SemanticColor::StatusCanceled), Color::Red);
    
    // Test priority colors
    assert_eq!(theme.get(SemanticColor::PriorityUrgent), Color::BrightRed);
    assert_eq!(theme.get(SemanticColor::PriorityHigh), Color::Red);
    assert_eq!(theme.get(SemanticColor::PriorityMedium), Color::Yellow);
}

#[test]
fn test_status_color_helper() {
    assert_eq!(status_color("completed"), SemanticColor::StatusCompleted);
    assert_eq!(status_color("done"), SemanticColor::StatusCompleted);
    assert_eq!(status_color("in_progress"), SemanticColor::StatusStarted);
    assert_eq!(status_color("started"), SemanticColor::StatusStarted);
    assert_eq!(status_color("canceled"), SemanticColor::StatusCanceled);
    assert_eq!(status_color("unknown"), SemanticColor::Primary);
}

#[test]
fn test_priority_color_helper() {
    assert_eq!(priority_color(0), SemanticColor::PriorityNone);
    assert_eq!(priority_color(1), SemanticColor::PriorityLow);
    assert_eq!(priority_color(2), SemanticColor::PriorityMedium);
    assert_eq!(priority_color(3), SemanticColor::PriorityHigh);
    assert_eq!(priority_color(4), SemanticColor::PriorityUrgent);
}

#[test]
fn test_priority_symbol_helper() {
    assert_eq!(priority_symbol(0), " ");
    assert_eq!(priority_symbol(1), "◦");
    assert_eq!(priority_symbol(2), "•");
    assert_eq!(priority_symbol(3), "■");
    assert_eq!(priority_symbol(4), "▲");
}

#[test]
fn test_themed_colorize() {
    let text = "Error message";
    let colored = text.with_theme(SemanticColor::Error);
    
    // The colored string should contain the text
    assert!(format!("{}", colored).contains("Error message"));
}

#[test]
fn test_theme_color_function() {
    let color = theme_color(SemanticColor::Success);
    assert_eq!(color, Color::Green);
    
    let color = theme_color(SemanticColor::Warning);
    assert_eq!(color, Color::Yellow);
}