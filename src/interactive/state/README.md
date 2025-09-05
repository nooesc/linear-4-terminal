# State Management System for Linear CLI

## Summary

I've created a new state management system for the Linear CLI's interactive mode that addresses the issues with the current monolithic `InteractiveApp` structure. The new system provides:

### Key Components Created:

1. **`state.rs`** - Core state management system
   - `ViewState` enum - Represents different UI views (IssueList, IssueDetail, etc.)
   - `InteractionMode` enum - How user input is handled (Normal, Search, Editing, Selecting)
   - `NavigationState` - Tracks position in lists and selected items
   - `EditMode` enum - What's being edited with associated data
   - `InputState` - Text input handling with cursor position
   - `AppState` - Complete application state
   - `StateCommand` enum - All possible state mutations
   - `StateMachine` - Manages transitions and history for undo/redo
   - `SideEffect` enum - External operations (API calls, etc.)

2. **`state_adapter.rs`** - Bridge between new and old systems
   - `StateAdapter` - Allows gradual migration from legacy code
   - Conversion functions between old and new state representations
   - Side effect handling that calls legacy methods

3. **`state_example.rs`** - Comprehensive usage examples
   - Basic navigation and editing workflows
   - Undo/redo demonstrations
   - Label selection patterns
   - Testing strategies

4. **`docs/state-management.md`** - Complete documentation
   - Architecture overview
   - Migration strategy
   - Usage examples
   - Future enhancements

## Benefits

1. **Separation of Concerns**: State, UI, and business logic are cleanly separated
2. **Immutable State Transitions**: All changes go through defined commands
3. **Undo/Redo Support**: Built-in history tracking
4. **Testability**: Pure functions for state transitions
5. **Type Safety**: Impossible states are unrepresentable
6. **Predictability**: Clear command → state → side effect flow

## Migration Path

The system is designed for gradual migration:

1. **Phase 1** (Current): Use `StateAdapter` to wrap existing code
2. **Phase 2**: Migrate individual commands and workflows
3. **Phase 3**: Replace `InteractiveApp` entirely

## Example Usage

```rust
// Create state machine
let mut state_machine = StateMachine::new(AppState::new());

// Navigate
state_machine.process_command(StateCommand::NavigateDown);

// Start editing
state_machine.process_command(StateCommand::StartEditingTitle);

// Type text
state_machine.process_command(StateCommand::InsertChar('H'));
state_machine.process_command(StateCommand::InsertChar('i'));

// Undo
state_machine.undo();
```

## Next Steps

To start using this system:

1. Integrate `StateAdapter` in `handlers.rs`
2. Start migrating simple commands (navigation)
3. Add tests for state transitions
4. Gradually move complex workflows
5. Eventually remove legacy `InteractiveApp`

The new system is ready to use and provides a much cleaner architecture for the interactive mode!