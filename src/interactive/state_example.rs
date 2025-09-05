/// Example demonstrating how to use the new state management system
/// 
/// This file shows how the new state system can be integrated into the Linear CLI,
/// providing better separation of concerns and enabling features like undo/redo.

use super::state::{
    AppState, StateCommand, StateMachine, ViewState, InteractionMode,
    EditMode, SideEffect, EditField, EditValue,
};
use super::state_adapter::StateAdapter;
use super::app::InteractiveApp;
use crossterm::event::KeyCode;

/// Example 1: Basic state transitions
fn example_basic_navigation() {
    let initial_state = AppState::new();
    let mut state_machine = StateMachine::new(initial_state);
    
    // Navigate down in the issue list
    let result = state_machine.process_command(StateCommand::NavigateDown);
    assert_eq!(result.new_state.navigation.issue_index, 1);
    
    // Enter detail view
    let result = state_machine.process_command(StateCommand::EnterDetailView);
    assert_eq!(result.new_state.view, ViewState::IssueDetail);
    
    // Start editing a comment
    let result = state_machine.process_command(StateCommand::StartEditingComment);
    assert_eq!(result.new_state.interaction, InteractionMode::Editing);
    
    // Type some text
    let result = state_machine.process_command(StateCommand::InsertChar('H'));
    let result = state_machine.process_command(StateCommand::InsertChar('i'));
    assert_eq!(result.new_state.input.content, "Hi");
    
    // Cancel editing
    let result = state_machine.process_command(StateCommand::CancelEdit);
    assert_eq!(result.new_state.interaction, InteractionMode::Normal);
    assert_eq!(result.new_state.input.content, ""); // Input cleared
}

/// Example 2: Undo/Redo functionality
fn example_undo_redo() {
    let initial_state = AppState::new();
    let mut state_machine = StateMachine::new(initial_state);
    
    // Make some changes
    state_machine.process_command(StateCommand::NavigateDown);
    state_machine.process_command(StateCommand::NavigateDown);
    state_machine.process_command(StateCommand::EnterDetailView);
    
    // Current state: Detail view, issue index 2
    assert_eq!(state_machine.current_state().view, ViewState::IssueDetail);
    assert_eq!(state_machine.current_state().navigation.issue_index, 2);
    
    // Undo last action (entering detail view)
    state_machine.undo();
    assert_eq!(state_machine.current_state().view, ViewState::IssueList);
    assert_eq!(state_machine.current_state().navigation.issue_index, 2);
    
    // Undo navigation
    state_machine.undo();
    assert_eq!(state_machine.current_state().navigation.issue_index, 1);
    
    // Redo
    state_machine.redo();
    assert_eq!(state_machine.current_state().navigation.issue_index, 2);
}

/// Example 3: Complex edit workflow with side effects
fn example_edit_workflow() {
    let mut initial_state = AppState::new();
    initial_state.navigation.selected_issue_id = Some("ISSUE-123".to_string());
    
    let mut state_machine = StateMachine::new(initial_state);
    
    // Start editing title
    let result = state_machine.process_command(StateCommand::StartEditingTitle);
    assert_eq!(result.new_state.interaction, InteractionMode::Editing);
    
    // Type new title
    state_machine.process_command(StateCommand::InsertChar('N'));
    state_machine.process_command(StateCommand::InsertChar('e'));
    state_machine.process_command(StateCommand::InsertChar('w'));
    
    // Submit edit (in real app, this would be triggered by Enter key)
    // This demonstrates how side effects are generated
    let edit_mode = state_machine.current_state().edit_mode.clone();
    if let EditMode::Title { issue_id, current_value } = edit_mode {
        // Generate submit side effect
        let side_effects = vec![
            SideEffect::SubmitEdit {
                issue_id,
                field: EditField::Title,
                value: EditValue::Text(current_value),
            }
        ];
        
        // The side effects would be handled by the adapter
        assert_eq!(side_effects.len(), 1);
    }
}

/// Example 4: Label selection workflow
fn example_label_selection() {
    let mut initial_state = AppState::new();
    initial_state.navigation.selected_issue_id = Some("ISSUE-123".to_string());
    
    let mut state_machine = StateMachine::new(initial_state);
    
    // Start editing labels with current labels
    let current_labels = vec!["label1".to_string(), "label2".to_string()];
    let result = state_machine.process_command(
        StateCommand::StartEditingLabels(current_labels)
    );
    
    assert_eq!(result.new_state.interaction, InteractionMode::Selecting);
    
    // Toggle a label selection
    state_machine.process_command(StateCommand::ToggleLabelSelection("label3".to_string()));
    
    // Check that label was added
    if let EditMode::Labels { selected_ids, .. } = &state_machine.current_state().edit_mode {
        assert_eq!(selected_ids.len(), 3);
        assert!(selected_ids.contains(&"label3".to_string()));
    }
    
    // Toggle existing label to remove it
    state_machine.process_command(StateCommand::ToggleLabelSelection("label1".to_string()));
    
    if let EditMode::Labels { selected_ids, .. } = &state_machine.current_state().edit_mode {
        assert_eq!(selected_ids.len(), 2);
        assert!(!selected_ids.contains(&"label1".to_string()));
    }
}

/// Example 5: Using the adapter with legacy code
async fn example_adapter_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create legacy app
    let legacy_app = InteractiveApp::new().await?;
    
    // Create adapter
    let mut adapter = StateAdapter::from_legacy_app(legacy_app).await?;
    
    // Handle key events through the new state system
    adapter.handle_key(KeyCode::Char('j')).await?; // Navigate down
    adapter.handle_key(KeyCode::Enter).await?;     // Enter detail view
    adapter.handle_key(KeyCode::Char('c')).await?; // Start comment
    
    // The adapter automatically syncs state back to the legacy app
    let legacy_app = adapter.legacy_app();
    assert_eq!(legacy_app.mode, super::app::AppMode::Comment);
    
    Ok(())
}

/// Example 6: Custom command sequences
fn example_command_macros() {
    let initial_state = AppState::new();
    let mut state_machine = StateMachine::new(initial_state);
    
    // Define a macro for "quick status change"
    let quick_status_change = vec![
        StateCommand::SelectIssue("ISSUE-123".to_string()),
        StateCommand::StartEditingStatus,
        StateCommand::NavigateDown, // Select next status
        StateCommand::NavigateDown, // Select next status
        // In real app, Enter would trigger submit
    ];
    
    // Execute the macro
    for command in quick_status_change {
        state_machine.process_command(command);
    }
    
    // Verify we're in status selection mode
    assert_eq!(state_machine.current_state().interaction, InteractionMode::Selecting);
    assert!(matches!(
        state_machine.current_state().edit_mode,
        EditMode::Status { .. }
    ));
}

/// Example 7: State persistence and restoration
fn example_state_persistence() {
    // Note: This would require adding Serialize/Deserialize to state structs
    // This is a conceptual example
    
    let mut state = AppState::new();
    state.navigation.issue_index = 5;
    state.search_query = "bug".to_string();
    state.hide_completed = true;
    
    // In a real implementation, you could serialize the state
    // let serialized = serde_json::to_string(&state).unwrap();
    
    // And later restore it
    // let restored_state: AppState = serde_json::from_str(&serialized).unwrap();
    
    // This enables features like:
    // - Saving and restoring sessions
    // - Sharing view states via URLs
    // - Implementing bookmarks
}

/// Example 8: Testing state transitions
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_navigation_bounds() {
        let initial_state = AppState::new();
        let mut state_machine = StateMachine::new(initial_state);
        
        // Navigate up when already at top
        let result = state_machine.process_command(StateCommand::NavigateUp);
        assert_eq!(result.new_state.navigation.issue_index, 0); // Should stay at 0
    }
    
    #[test]
    fn test_edit_mode_transitions() {
        let mut initial_state = AppState::new();
        initial_state.navigation.selected_issue_id = Some("TEST-1".to_string());
        let mut state_machine = StateMachine::new(initial_state);
        
        // Cannot start editing without selected issue
        let mut state_without_issue = AppState::new();
        state_without_issue.navigation.selected_issue_id = None;
        let mut machine = StateMachine::new(state_without_issue);
        
        let result = machine.process_command(StateCommand::StartEditingTitle);
        assert_eq!(result.new_state.edit_mode, EditMode::None); // Should not change
    }
    
    #[test]
    fn test_search_workflow() {
        let initial_state = AppState::new();
        let mut state_machine = StateMachine::new(initial_state);
        
        // Start search
        state_machine.process_command(StateCommand::StartSearch);
        
        // Type search query
        state_machine.process_command(StateCommand::InsertChar('b'));
        state_machine.process_command(StateCommand::InsertChar('u'));
        state_machine.process_command(StateCommand::InsertChar('g'));
        
        assert_eq!(state_machine.current_state().search_query, "bug");
        
        // Clear search
        let result = state_machine.process_command(StateCommand::ClearSearch);
        assert_eq!(result.new_state.search_query, "");
        assert_eq!(result.new_state.interaction, InteractionMode::Normal);
        
        // Should have refresh side effect
        assert!(result.side_effects.iter().any(|e| matches!(e, SideEffect::RefreshIssues)));
    }
}

/// Benefits of the new state system:
/// 
/// 1. **Separation of Concerns**: State, UI, and business logic are clearly separated
/// 2. **Testability**: State transitions can be tested without UI or async operations
/// 3. **Undo/Redo**: History tracking enables undo/redo functionality
/// 4. **Predictability**: All state changes go through defined commands
/// 5. **Debugging**: State transitions can be logged and replayed
/// 6. **Extensibility**: New commands and states can be added without changing existing code
/// 7. **Type Safety**: Impossible states are unrepresentable
/// 8. **Side Effect Management**: Clear separation between state changes and effects
/// 
/// Migration strategy:
/// 1. Use StateAdapter to gradually migrate functionality
/// 2. Start with simple commands (navigation, view changes)
/// 3. Move complex workflows (editing, API calls) piece by piece
/// 4. Eventually replace InteractiveApp with pure state-based implementation