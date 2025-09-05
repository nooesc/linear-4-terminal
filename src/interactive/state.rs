use std::collections::VecDeque;
use crossterm::event::KeyCode;

/// Represents the current view state of the application
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewState {
    /// Normal browsing mode - viewing list of issues
    IssueList,
    /// Viewing detailed information about a single issue
    IssueDetail,
    /// Browsing links within an issue
    LinkNavigation,
    /// External editor is active
    ExternalEditor,
}

/// Represents navigation position within lists and menus
#[derive(Debug, Clone)]
pub struct NavigationState {
    /// Currently selected index in the issue list
    pub issue_index: usize,
    /// Currently selected link index
    pub link_index: usize,
    /// Currently selected option in menus
    pub option_index: usize,
    /// Scroll offset for long lists
    pub scroll_offset: usize,
    /// Currently selected issue ID (if any)
    pub selected_issue_id: Option<String>,
}

impl NavigationState {
    pub fn new() -> Self {
        Self {
            issue_index: 0,
            link_index: 0,
            option_index: 0,
            scroll_offset: 0,
            selected_issue_id: None,
        }
    }

    pub fn reset_indices(&mut self) {
        self.issue_index = 0;
        self.link_index = 0;
        self.option_index = 0;
        self.scroll_offset = 0;
    }
}

/// Represents what is currently being edited
#[derive(Debug, Clone, PartialEq)]
pub enum EditMode {
    /// Not editing anything
    None,
    /// Editing issue title
    Title { issue_id: String, current_value: String },
    /// Editing issue description
    Description { issue_id: String, current_value: String },
    /// Selecting a new status
    Status { issue_id: String },
    /// Selecting a new assignee
    Assignee { issue_id: String },
    /// Selecting priority
    Priority { issue_id: String },
    /// Selecting labels
    Labels { issue_id: String, selected_ids: Vec<String> },
    /// Selecting project
    Project { issue_id: String, current_id: Option<String> },
    /// Writing a comment
    Comment { issue_id: String, text: String },
}

/// Input state for text editing
#[derive(Debug, Clone)]
pub struct InputState {
    /// Current text content
    pub content: String,
    /// Cursor position within the text
    pub cursor_position: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
        }
    }

    pub fn from_content(content: String) -> Self {
        let cursor_position = content.len();
        Self {
            content,
            cursor_position,
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.content.remove(self.cursor_position);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.content.len() {
            self.cursor_position += 1;
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.content.len();
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor_position = 0;
    }
}

/// Represents UI interaction modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionMode {
    /// Normal navigation mode
    Normal,
    /// Searching/filtering issues
    Search,
    /// Editing a field
    Editing,
    /// Selecting from options
    Selecting,
}

/// Represents the complete application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current view being displayed
    pub view: ViewState,
    /// Current interaction mode
    pub interaction: InteractionMode,
    /// Navigation state
    pub navigation: NavigationState,
    /// Current edit mode (if any)
    pub edit_mode: EditMode,
    /// Input state for text fields
    pub input: InputState,
    /// Search/filter query
    pub search_query: String,
    /// Whether to hide completed issues
    pub hide_completed: bool,
    /// Group by mode
    pub group_by: super::app::GroupBy,
    /// Error message to display
    pub error_message: Option<String>,
    /// Loading state
    pub loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            view: ViewState::IssueList,
            interaction: InteractionMode::Normal,
            navigation: NavigationState::new(),
            edit_mode: EditMode::None,
            input: InputState::new(),
            search_query: String::new(),
            hide_completed: false,
            group_by: super::app::GroupBy::Status,
            error_message: None,
            loading: false,
        }
    }

    /// Check if the app is in any editing state
    pub fn is_editing(&self) -> bool {
        !matches!(self.edit_mode, EditMode::None) || self.interaction == InteractionMode::Search
    }

    /// Get the current issue ID being operated on
    pub fn current_issue_id(&self) -> Option<&str> {
        match &self.edit_mode {
            EditMode::None => self.navigation.selected_issue_id.as_deref(),
            EditMode::Title { issue_id, .. } |
            EditMode::Description { issue_id, .. } |
            EditMode::Status { issue_id } |
            EditMode::Assignee { issue_id } |
            EditMode::Priority { issue_id } |
            EditMode::Labels { issue_id, .. } |
            EditMode::Project { issue_id, .. } |
            EditMode::Comment { issue_id, .. } => Some(issue_id),
        }
    }
}

/// Commands that can mutate the application state
#[derive(Debug, Clone)]
pub enum StateCommand {
    // Navigation commands
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    SelectIssue(String),
    
    // View transitions
    EnterDetailView,
    ExitDetailView,
    EnterLinkNavigation,
    ExitLinkNavigation,
    
    // Edit mode transitions
    StartEditingTitle,
    StartEditingDescription,
    StartEditingStatus,
    StartEditingPriority,
    StartEditingLabels(Vec<String>), // Current label IDs
    StartEditingProject(Option<String>), // Current project ID
    StartEditingComment,
    CancelEdit,
    
    // Text input commands
    InsertChar(char),
    DeleteChar,
    Backspace,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    
    // Search/filter commands
    StartSearch,
    UpdateSearchQuery(String),
    ClearSearch,
    ToggleHideCompleted,
    
    // Other commands
    SetError(String),
    ClearError,
    SetLoading(bool),
    ToggleGroupBy,
    
    // Label selection
    ToggleLabelSelection(String),
    
    // External editor
    LaunchExternalEditor,
    ReturnFromExternalEditor(Option<String>), // New content
}

/// Result of a state transition
#[derive(Debug)]
pub struct TransitionResult {
    pub new_state: AppState,
    pub side_effects: Vec<SideEffect>,
}

/// Side effects that need to be handled after state transitions
#[derive(Debug, Clone)]
pub enum SideEffect {
    /// Refresh the issue list from the API
    RefreshIssues,
    /// Submit an edit to the API
    SubmitEdit {
        issue_id: String,
        field: EditField,
        value: EditValue,
    },
    /// Submit a comment to the API
    SubmitComment {
        issue_id: String,
        text: String,
    },
    /// Open a URL in the browser
    OpenUrl(String),
    /// Launch external editor with content
    LaunchEditor(String),
    /// Exit the application
    Quit,
}

#[derive(Debug, Clone)]
pub enum EditField {
    Title,
    Description,
    Status,
    Priority,
    Labels,
    Project,
}

#[derive(Debug, Clone)]
pub enum EditValue {
    Text(String),
    Status(String),
    Priority(u8),
    Labels(Vec<String>),
    Project(Option<String>),
}

/// State machine that handles transitions
pub struct StateMachine {
    /// History for undo/redo support
    history: VecDeque<AppState>,
    /// Maximum history size
    max_history: usize,
    /// Current position in history
    history_position: usize,
}

impl StateMachine {
    pub fn new(initial_state: AppState) -> Self {
        let mut history = VecDeque::with_capacity(100);
        history.push_back(initial_state);
        
        Self {
            history,
            max_history: 100,
            history_position: 0,
        }
    }

    /// Get the current state
    pub fn current_state(&self) -> &AppState {
        &self.history[self.history_position]
    }

    /// Process a command and return the new state with side effects
    pub fn process_command(&mut self, command: StateCommand) -> TransitionResult {
        let current = self.current_state().clone();
        let (new_state, side_effects) = apply_transition(current, command);
        
        // Add to history if state changed
        if self.should_record_in_history(&new_state) {
            // Remove any states after current position (for redo)
            self.history.truncate(self.history_position + 1);
            
            // Add new state
            self.history.push_back(new_state.clone());
            
            // Maintain max history size
            if self.history.len() > self.max_history {
                self.history.pop_front();
            } else {
                self.history_position += 1;
            }
        }
        
        TransitionResult {
            new_state,
            side_effects,
        }
    }

    /// Check if a state change should be recorded in history
    fn should_record_in_history(&self, new_state: &AppState) -> bool {
        let current = self.current_state();
        
        // Don't record cursor movements or temporary states
        match (&current.edit_mode, &new_state.edit_mode) {
            (EditMode::Title { current_value: old, .. }, EditMode::Title { current_value: new, .. }) |
            (EditMode::Description { current_value: old, .. }, EditMode::Description { current_value: new, .. }) |
            (EditMode::Comment { text: old, .. }, EditMode::Comment { text: new, .. }) => {
                // Only record if actual text changed (not just cursor movement)
                old != new
            }
            _ => {
                // Record view changes, mode changes, etc.
                current.view != new_state.view || 
                current.interaction != new_state.interaction ||
                current.edit_mode != new_state.edit_mode
            }
        }
    }

    /// Undo the last action
    pub fn undo(&mut self) -> Option<&AppState> {
        if self.history_position > 0 {
            self.history_position -= 1;
            Some(self.current_state())
        } else {
            None
        }
    }

    /// Redo the next action
    pub fn redo(&mut self) -> Option<&AppState> {
        if self.history_position + 1 < self.history.len() {
            self.history_position += 1;
            Some(self.current_state())
        } else {
            None
        }
    }
}

/// Apply a state transition based on the command
fn apply_transition(mut state: AppState, command: StateCommand) -> (AppState, Vec<SideEffect>) {
    let mut side_effects = Vec::new();
    
    match command {
        // Navigation commands
        StateCommand::NavigateUp => {
            if state.interaction == InteractionMode::Normal {
                if state.navigation.issue_index > 0 {
                    state.navigation.issue_index -= 1;
                }
            } else if state.interaction == InteractionMode::Selecting {
                if state.navigation.option_index > 0 {
                    state.navigation.option_index -= 1;
                }
            }
        }
        
        StateCommand::NavigateDown => {
            if state.interaction == InteractionMode::Normal {
                state.navigation.issue_index += 1; // Bounds checking done elsewhere
            } else if state.interaction == InteractionMode::Selecting {
                state.navigation.option_index += 1; // Bounds checking done elsewhere
            }
        }
        
        StateCommand::NavigateLeft | StateCommand::NavigateRight => {
            // These are primarily for cursor movement within text fields
            // Currently handled by MoveCursor commands
        }
        
        StateCommand::SelectIssue(issue_id) => {
            state.navigation.selected_issue_id = Some(issue_id);
        }
        
        // View transitions
        StateCommand::EnterDetailView => {
            if state.view == ViewState::IssueList {
                state.view = ViewState::IssueDetail;
            }
        }
        
        StateCommand::ExitDetailView => {
            if state.view == ViewState::IssueDetail {
                state.view = ViewState::IssueList;
            }
        }
        
        StateCommand::EnterLinkNavigation => {
            if state.view == ViewState::IssueDetail {
                state.view = ViewState::LinkNavigation;
                state.navigation.link_index = 0;
            }
        }
        
        StateCommand::ExitLinkNavigation => {
            if state.view == ViewState::LinkNavigation {
                state.view = ViewState::IssueDetail;
            }
        }
        
        // Edit mode transitions
        StateCommand::StartEditingTitle => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Title {
                    issue_id,
                    current_value: state.input.content.clone(),
                };
                state.interaction = InteractionMode::Editing;
            }
        }
        
        StateCommand::StartEditingDescription => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Description {
                    issue_id,
                    current_value: state.input.content.clone(),
                };
                state.interaction = InteractionMode::Editing;
            }
        }
        
        StateCommand::StartEditingStatus => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Status { issue_id };
                state.interaction = InteractionMode::Selecting;
                state.navigation.option_index = 0;
            }
        }
        
        StateCommand::StartEditingPriority => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Priority { issue_id };
                state.interaction = InteractionMode::Selecting;
                state.navigation.option_index = 0;
            }
        }
        
        StateCommand::StartEditingLabels(current_labels) => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Labels {
                    issue_id,
                    selected_ids: current_labels,
                };
                state.interaction = InteractionMode::Selecting;
                state.navigation.option_index = 0;
            }
        }
        
        StateCommand::StartEditingProject(current_project) => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Project {
                    issue_id,
                    current_id: current_project,
                };
                state.interaction = InteractionMode::Selecting;
                state.navigation.option_index = 0;
            }
        }
        
        StateCommand::StartEditingComment => {
            if let Some(issue_id) = state.navigation.selected_issue_id.clone() {
                state.edit_mode = EditMode::Comment {
                    issue_id,
                    text: String::new(),
                };
                state.interaction = InteractionMode::Editing;
                state.input.clear();
            }
        }
        
        StateCommand::CancelEdit => {
            state.edit_mode = EditMode::None;
            state.interaction = InteractionMode::Normal;
            state.input.clear();
        }
        
        // Text input commands
        StateCommand::InsertChar(ch) => {
            if state.interaction == InteractionMode::Editing {
                state.input.insert_char(ch);
                // Update edit mode content
                match &mut state.edit_mode {
                    EditMode::Title { current_value, .. } => {
                        *current_value = state.input.content.clone();
                    }
                    EditMode::Description { current_value, .. } => {
                        *current_value = state.input.content.clone();
                    }
                    EditMode::Comment { text, .. } => {
                        *text = state.input.content.clone();
                    }
                    _ => {}
                }
            } else if state.interaction == InteractionMode::Search {
                state.search_query.push(ch);
                side_effects.push(SideEffect::RefreshIssues);
            }
        }
        
        StateCommand::Backspace => {
            if state.interaction == InteractionMode::Editing {
                state.input.backspace();
                // Update edit mode content
                match &mut state.edit_mode {
                    EditMode::Title { current_value, .. } => {
                        *current_value = state.input.content.clone();
                    }
                    EditMode::Description { current_value, .. } => {
                        *current_value = state.input.content.clone();
                    }
                    EditMode::Comment { text, .. } => {
                        *text = state.input.content.clone();
                    }
                    _ => {}
                }
            } else if state.interaction == InteractionMode::Search {
                state.search_query.pop();
                side_effects.push(SideEffect::RefreshIssues);
            }
        }
        
        StateCommand::DeleteChar => {
            if state.interaction == InteractionMode::Editing {
                state.input.delete_char();
            }
        }
        
        StateCommand::MoveCursorLeft => {
            if state.interaction == InteractionMode::Editing {
                state.input.move_cursor_left();
            }
        }
        
        StateCommand::MoveCursorRight => {
            if state.interaction == InteractionMode::Editing {
                state.input.move_cursor_right();
            }
        }
        
        StateCommand::MoveCursorHome => {
            if state.interaction == InteractionMode::Editing {
                state.input.move_cursor_home();
            }
        }
        
        StateCommand::MoveCursorEnd => {
            if state.interaction == InteractionMode::Editing {
                state.input.move_cursor_end();
            }
        }
        
        // Search commands
        StateCommand::StartSearch => {
            state.interaction = InteractionMode::Search;
        }
        
        StateCommand::UpdateSearchQuery(query) => {
            state.search_query = query;
            side_effects.push(SideEffect::RefreshIssues);
        }
        
        StateCommand::ClearSearch => {
            state.search_query.clear();
            state.interaction = InteractionMode::Normal;
            side_effects.push(SideEffect::RefreshIssues);
        }
        
        StateCommand::ToggleHideCompleted => {
            state.hide_completed = !state.hide_completed;
            side_effects.push(SideEffect::RefreshIssues);
        }
        
        // Other commands
        StateCommand::SetError(msg) => {
            state.error_message = Some(msg);
        }
        
        StateCommand::ClearError => {
            state.error_message = None;
        }
        
        StateCommand::SetLoading(loading) => {
            state.loading = loading;
        }
        
        StateCommand::ToggleGroupBy => {
            state.group_by = match state.group_by {
                super::app::GroupBy::Status => super::app::GroupBy::Project,
                super::app::GroupBy::Project => super::app::GroupBy::Status,
            };
            side_effects.push(SideEffect::RefreshIssues);
        }
        
        StateCommand::ToggleLabelSelection(label_id) => {
            if let EditMode::Labels { selected_ids, .. } = &mut state.edit_mode {
                if let Some(pos) = selected_ids.iter().position(|id| id == &label_id) {
                    selected_ids.remove(pos);
                } else {
                    selected_ids.push(label_id);
                }
            }
        }
        
        StateCommand::LaunchExternalEditor => {
            if let EditMode::Description { current_value, .. } = &state.edit_mode {
                state.view = ViewState::ExternalEditor;
                side_effects.push(SideEffect::LaunchEditor(current_value.clone()));
            }
        }
        
        StateCommand::ReturnFromExternalEditor(content) => {
            state.view = ViewState::IssueDetail;
            if let Some(new_content) = content {
                state.input = InputState::from_content(new_content.clone());
                if let EditMode::Description { current_value, .. } = &mut state.edit_mode {
                    *current_value = new_content;
                }
            }
        }
    }
    
    (state, side_effects)
}

/// Map keyboard input to state commands
pub fn map_key_to_command(key: KeyCode, state: &AppState) -> Option<StateCommand> {
    match (state.interaction, state.view, key) {
        // Normal mode navigation
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Char('j')) |
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Down) => {
            Some(StateCommand::NavigateDown)
        }
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Char('k')) |
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Up) => {
            Some(StateCommand::NavigateUp)
        }
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Enter) => {
            Some(StateCommand::EnterDetailView)
        }
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Char('/')) => {
            Some(StateCommand::StartSearch)
        }
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Char('d')) => {
            Some(StateCommand::ToggleHideCompleted)
        }
        (InteractionMode::Normal, ViewState::IssueList, KeyCode::Char('g')) => {
            Some(StateCommand::ToggleGroupBy)
        }
        
        // Detail view
        (InteractionMode::Normal, ViewState::IssueDetail, KeyCode::Esc) |
        (InteractionMode::Normal, ViewState::IssueDetail, KeyCode::Char('q')) => {
            Some(StateCommand::ExitDetailView)
        }
        (InteractionMode::Normal, ViewState::IssueDetail, KeyCode::Char('l')) => {
            Some(StateCommand::EnterLinkNavigation)
        }
        (InteractionMode::Normal, ViewState::IssueDetail, KeyCode::Char('c')) => {
            Some(StateCommand::StartEditingComment)
        }
        
        // Link navigation
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Esc) |
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Char('q')) => {
            Some(StateCommand::ExitLinkNavigation)
        }
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Char('j')) |
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Down) => {
            Some(StateCommand::NavigateDown)
        }
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Char('k')) |
        (InteractionMode::Normal, ViewState::LinkNavigation, KeyCode::Up) => {
            Some(StateCommand::NavigateUp)
        }
        
        // Search mode
        (InteractionMode::Search, _, KeyCode::Esc) => {
            Some(StateCommand::ClearSearch)
        }
        (InteractionMode::Search, _, KeyCode::Enter) => {
            Some(StateCommand::UpdateSearchQuery(state.search_query.clone()))
        }
        (InteractionMode::Search, _, KeyCode::Char(ch)) => {
            Some(StateCommand::InsertChar(ch))
        }
        (InteractionMode::Search, _, KeyCode::Backspace) => {
            Some(StateCommand::Backspace)
        }
        
        // Editing mode
        (InteractionMode::Editing, _, KeyCode::Esc) => {
            Some(StateCommand::CancelEdit)
        }
        (InteractionMode::Editing, _, KeyCode::Char(ch)) => {
            Some(StateCommand::InsertChar(ch))
        }
        (InteractionMode::Editing, _, KeyCode::Backspace) => {
            Some(StateCommand::Backspace)
        }
        (InteractionMode::Editing, _, KeyCode::Delete) => {
            Some(StateCommand::DeleteChar)
        }
        (InteractionMode::Editing, _, KeyCode::Left) => {
            Some(StateCommand::MoveCursorLeft)
        }
        (InteractionMode::Editing, _, KeyCode::Right) => {
            Some(StateCommand::MoveCursorRight)
        }
        (InteractionMode::Editing, _, KeyCode::Home) => {
            Some(StateCommand::MoveCursorHome)
        }
        (InteractionMode::Editing, _, KeyCode::End) => {
            Some(StateCommand::MoveCursorEnd)
        }
        
        // Selecting mode
        (InteractionMode::Selecting, _, KeyCode::Esc) |
        (InteractionMode::Selecting, _, KeyCode::Char('q')) => {
            Some(StateCommand::CancelEdit)
        }
        (InteractionMode::Selecting, _, KeyCode::Char('j')) |
        (InteractionMode::Selecting, _, KeyCode::Down) => {
            Some(StateCommand::NavigateDown)
        }
        (InteractionMode::Selecting, _, KeyCode::Char('k')) |
        (InteractionMode::Selecting, _, KeyCode::Up) => {
            Some(StateCommand::NavigateUp)
        }
        
        _ => None,
    }
}

/// Quick edit shortcuts that can be triggered from various states
pub fn get_quick_edit_command(key: KeyCode, state: &AppState) -> Option<StateCommand> {
    match key {
        KeyCode::Char('s') if state.interaction == InteractionMode::Normal => {
            Some(StateCommand::StartEditingStatus)
        }
        KeyCode::Char('c') if state.interaction == InteractionMode::Normal => {
            Some(StateCommand::StartEditingComment)
        }
        KeyCode::Char('l') if state.interaction == InteractionMode::Normal && state.view == ViewState::IssueList => {
            // Need to pass current labels - this would be handled by the app
            Some(StateCommand::StartEditingLabels(vec![]))
        }
        KeyCode::Char('p') if state.interaction == InteractionMode::Normal => {
            Some(StateCommand::StartEditingProject(None))
        }
        _ => None,
    }
}