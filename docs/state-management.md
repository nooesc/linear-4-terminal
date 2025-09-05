# State Management System Documentation

## Overview

The new state management system for Linear CLI's interactive mode provides a clean separation of concerns, improved testability, and support for advanced features like undo/redo. This document explains the architecture and migration path.

## Architecture Components

### 1. Core State Types

```rust
// View states represent what screen is shown
pub enum ViewState {
    IssueList,      // Main issue list view
    IssueDetail,    // Detailed issue view
    LinkNavigation, // Navigating links within an issue
    ExternalEditor, // External editor is active
}

// Interaction modes represent how user input is handled
pub enum InteractionMode {
    Normal,    // Navigation and shortcuts
    Search,    // Text input for search
    Editing,   // Text input for fields
    Selecting, // Selecting from options
}

// Edit modes represent what's being edited
pub enum EditMode {
    None,
    Title { issue_id: String, current_value: String },
    Description { issue_id: String, current_value: String },
    Status { issue_id: String },
    // ... etc
}
```

### 2. State Machine

The `StateMachine` manages state transitions and history:

- Processes commands to produce new states
- Maintains history for undo/redo
- Generates side effects for async operations

### 3. Commands and Side Effects

All state changes happen through commands:

```rust
pub enum StateCommand {
    NavigateUp,
    NavigateDown,
    StartEditingTitle,
    InsertChar(char),
    // ... etc
}
```

Side effects represent operations that need to happen outside the state system:

```rust
pub enum SideEffect {
    RefreshIssues,
    SubmitEdit { issue_id: String, field: EditField, value: EditValue },
    OpenUrl(String),
    // ... etc
}
```

## Benefits

1. **Testability**: State transitions can be tested without UI or async code
2. **Predictability**: All changes go through defined commands
3. **History**: Built-in undo/redo support
4. **Debugging**: State changes can be logged and replayed
5. **Type Safety**: Impossible states are unrepresentable

## Migration Strategy

### Phase 1: Adapter Pattern (Current)

The `StateAdapter` bridges the new system with existing code:

```rust
// In handlers.rs
let mut adapter = StateAdapter::from_legacy_app(app).await?;
adapter.handle_key(key).await?;
```

### Phase 2: Gradual Function Migration

1. Start with simple navigation commands
2. Move view transitions
3. Migrate edit workflows
4. Port async operations

### Phase 3: Full Migration

1. Replace `InteractiveApp` with pure state-based implementation
2. Update UI to read from new state
3. Remove legacy code

## Usage Examples

### Basic Navigation

```rust
let mut state_machine = StateMachine::new(AppState::new());

// Navigate in issue list
state_machine.process_command(StateCommand::NavigateDown);
state_machine.process_command(StateCommand::EnterDetailView);
```

### Editing Workflow

```rust
// Start editing
state_machine.process_command(StateCommand::StartEditingTitle);

// Type text
state_machine.process_command(StateCommand::InsertChar('H'));
state_machine.process_command(StateCommand::InsertChar('i'));

// Submit generates side effect
let result = state_machine.process_command(StateCommand::SubmitEdit);
// result.side_effects contains SubmitEdit effect
```

### Undo/Redo

```rust
// Make changes
state_machine.process_command(StateCommand::NavigateDown);
state_machine.process_command(StateCommand::EnterDetailView);

// Undo last action
state_machine.undo();

// Redo
state_machine.redo();
```

## Testing

The new system makes testing much easier:

```rust
#[test]
fn test_navigation() {
    let mut state = AppState::new();
    let (new_state, effects) = apply_transition(state, StateCommand::NavigateDown);
    assert_eq!(new_state.navigation.issue_index, 1);
    assert!(effects.is_empty());
}
```

## Future Enhancements

1. **State Persistence**: Save/restore sessions
2. **Macros**: Record and replay command sequences
3. **Remote State**: Share state via URLs
4. **Time Travel Debugging**: Step through state history
5. **State Validation**: Ensure state consistency

## Implementation Checklist

- [x] Core state types defined
- [x] State machine with history
- [x] Command processing
- [x] Side effect generation
- [x] Adapter for legacy code
- [ ] Migrate navigation commands
- [ ] Migrate edit workflows
- [ ] Update UI components
- [ ] Add state persistence
- [ ] Complete migration