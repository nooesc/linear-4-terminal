use super::app::{InteractiveApp, AppMode, EditField as LegacyEditField};
use super::state::{
    AppState, StateCommand, StateMachine, TransitionResult, SideEffect,
    ViewState, InteractionMode, EditMode, EditField, EditValue,
};
use crossterm::event::KeyCode;
use std::error::Error;

/// Adapter that bridges the new state system with the existing InteractiveApp
pub struct StateAdapter {
    /// The new state machine
    state_machine: StateMachine,
    /// Reference to the legacy app (for data access)
    legacy_app: InteractiveApp,
}

impl StateAdapter {
    /// Create a new adapter from an existing InteractiveApp
    pub async fn from_legacy_app(app: InteractiveApp) -> Result<Self, Box<dyn Error>> {
        // Convert legacy state to new state
        let initial_state = convert_legacy_to_new_state(&app);
        let state_machine = StateMachine::new(initial_state);
        
        Ok(Self {
            state_machine,
            legacy_app: app,
        })
    }
    
    /// Handle a key event using the new state system
    pub async fn handle_key(&mut self, key: KeyCode) -> Result<(), Box<dyn Error>> {
        let current_state = self.state_machine.current_state();
        
        // Map key to command
        let command = if let Some(cmd) = super::state::map_key_to_command(key, current_state) {
            cmd
        } else if let Some(cmd) = super::state::get_quick_edit_command(key, current_state) {
            // Handle quick edit commands, injecting current data
            match cmd {
                StateCommand::StartEditingLabels(_) => {
                    // Get current labels from selected issue
                    if let Some(issue) = self.legacy_app.get_selected_issue() {
                        let label_ids: Vec<String> = issue.labels.nodes.iter()
                            .map(|label| label.id.clone())
                            .collect();
                        StateCommand::StartEditingLabels(label_ids)
                    } else {
                        return Ok(());
                    }
                }
                StateCommand::StartEditingProject(_) => {
                    // Get current project from selected issue
                    if let Some(issue) = self.legacy_app.get_selected_issue() {
                        let project_id = issue.project.as_ref().map(|p| p.id.clone());
                        StateCommand::StartEditingProject(project_id)
                    } else {
                        return Ok(());
                    }
                }
                _ => cmd,
            }
        } else {
            // No command for this key in current state
            return Ok(());
        };
        
        // Process the command
        let TransitionResult { new_state, side_effects } = self.state_machine.process_command(command);
        
        // Sync state back to legacy app
        sync_state_to_legacy(&new_state, &mut self.legacy_app);
        
        // Handle side effects
        for effect in side_effects {
            self.handle_side_effect(effect).await?;
        }
        
        Ok(())
    }
    
    /// Handle side effects from state transitions
    async fn handle_side_effect(&mut self, effect: SideEffect) -> Result<(), Box<dyn Error>> {
        match effect {
            SideEffect::RefreshIssues => {
                self.legacy_app.apply_filters();
            }
            
            SideEffect::SubmitEdit { issue_id, field, value } => {
                self.legacy_app.selected_issue_id = Some(issue_id.clone());
                
                match (field, value) {
                    (EditField::Title, EditValue::Text(text)) => {
                        self.legacy_app.edit_field = LegacyEditField::Title;
                        self.legacy_app.edit_input = text;
                    }
                    (EditField::Description, EditValue::Text(text)) => {
                        self.legacy_app.edit_field = LegacyEditField::Description;
                        self.legacy_app.edit_input = text;
                    }
                    (EditField::Status, EditValue::Status(status_id)) => {
                        self.legacy_app.edit_field = LegacyEditField::Status;
                        self.legacy_app.selected_option = Some(status_id);
                    }
                    (EditField::Priority, EditValue::Priority(priority)) => {
                        self.legacy_app.edit_field = LegacyEditField::Priority;
                        self.legacy_app.selected_option = Some(priority.to_string());
                    }
                    (EditField::Labels, EditValue::Labels(label_ids)) => {
                        self.legacy_app.edit_field = LegacyEditField::Labels;
                        self.legacy_app.selected_labels = label_ids;
                    }
                    (EditField::Project, EditValue::Project(project_id)) => {
                        self.legacy_app.edit_field = LegacyEditField::Project;
                        self.legacy_app.selected_option = project_id.map(|id| {
                            if id.is_empty() { "none".to_string() } else { id }
                        });
                    }
                    _ => {}
                }
                
                self.legacy_app.submit_edit().await?;
            }
            
            SideEffect::SubmitComment { issue_id, text } => {
                self.legacy_app.selected_issue_id = Some(issue_id);
                self.legacy_app.comment_input = text;
                self.legacy_app.submit_comment().await?;
            }
            
            SideEffect::OpenUrl(url) => {
                let _ = self.legacy_app.open_link(&url);
            }
            
            SideEffect::LaunchEditor(content) => {
                self.legacy_app.edit_input = content;
                self.legacy_app.mode = AppMode::ExternalEditor;
            }
            
            SideEffect::Quit => {
                self.legacy_app.should_quit = true;
            }
        }
        
        Ok(())
    }
    
    /// Get a reference to the legacy app (for UI rendering)
    pub fn legacy_app(&self) -> &InteractiveApp {
        &self.legacy_app
    }
    
    /// Get a mutable reference to the legacy app
    pub fn legacy_app_mut(&mut self) -> &mut InteractiveApp {
        &mut self.legacy_app
    }
}

/// Convert legacy AppMode to new ViewState
fn legacy_mode_to_view_state(mode: AppMode) -> ViewState {
    match mode {
        AppMode::Normal | AppMode::Search | AppMode::Filter => ViewState::IssueList,
        AppMode::Detail | AppMode::Comment | AppMode::Edit | AppMode::EditField | AppMode::SelectOption => ViewState::IssueDetail,
        AppMode::Links => ViewState::LinkNavigation,
        AppMode::ExternalEditor => ViewState::ExternalEditor,
    }
}

/// Convert legacy AppMode to new InteractionMode
fn legacy_mode_to_interaction_mode(mode: AppMode) -> InteractionMode {
    match mode {
        AppMode::Normal | AppMode::Detail | AppMode::Links => InteractionMode::Normal,
        AppMode::Search | AppMode::Filter => InteractionMode::Search,
        AppMode::Comment | AppMode::EditField => InteractionMode::Editing,
        AppMode::Edit | AppMode::SelectOption => InteractionMode::Selecting,
        AppMode::ExternalEditor => InteractionMode::Normal,
    }
}

/// Convert legacy state to new AppState
fn convert_legacy_to_new_state(app: &InteractiveApp) -> AppState {
    let mut state = AppState::new();
    
    // Convert view state
    state.view = legacy_mode_to_view_state(app.mode);
    state.interaction = legacy_mode_to_interaction_mode(app.mode);
    
    // Convert navigation state
    state.navigation.issue_index = app.selected_index;
    state.navigation.link_index = app.selected_link_index;
    state.navigation.option_index = app.option_index;
    state.navigation.selected_issue_id = app.selected_issue_id.clone();
    
    // Convert edit mode
    state.edit_mode = match app.mode {
        AppMode::Comment => {
            if let Some(issue_id) = &app.selected_issue_id {
                EditMode::Comment {
                    issue_id: issue_id.clone(),
                    text: app.comment_input.clone(),
                }
            } else {
                EditMode::None
            }
        }
        AppMode::EditField => {
            if let Some(issue_id) = &app.selected_issue_id {
                match app.edit_field {
                    LegacyEditField::Title => EditMode::Title {
                        issue_id: issue_id.clone(),
                        current_value: app.edit_input.clone(),
                    },
                    LegacyEditField::Description => EditMode::Description {
                        issue_id: issue_id.clone(),
                        current_value: app.edit_input.clone(),
                    },
                    _ => EditMode::None,
                }
            } else {
                EditMode::None
            }
        }
        AppMode::SelectOption => {
            if let Some(issue_id) = &app.selected_issue_id {
                match app.edit_field {
                    LegacyEditField::Status => EditMode::Status { issue_id: issue_id.clone() },
                    LegacyEditField::Priority => EditMode::Priority { issue_id: issue_id.clone() },
                    LegacyEditField::Labels => EditMode::Labels {
                        issue_id: issue_id.clone(),
                        selected_ids: app.selected_labels.clone(),
                    },
                    LegacyEditField::Project => EditMode::Project {
                        issue_id: issue_id.clone(),
                        current_id: app.selected_option.clone().filter(|s| s != "none"),
                    },
                    _ => EditMode::None,
                }
            } else {
                EditMode::None
            }
        }
        _ => EditMode::None,
    };
    
    // Convert input state
    state.input.content = match app.mode {
        AppMode::Comment => app.comment_input.clone(),
        AppMode::EditField => app.edit_input.clone(),
        _ => String::new(),
    };
    state.input.cursor_position = match app.mode {
        AppMode::Comment => app.comment_cursor_position,
        AppMode::EditField => app.cursor_position,
        _ => 0,
    };
    
    // Convert other state
    state.search_query = app.search_query.clone();
    state.hide_completed = app.hide_done_issues;
    state.group_by = app.group_by;
    state.error_message = app.error_message.clone();
    state.loading = app.loading;
    
    state
}

/// Sync new state back to legacy app
fn sync_state_to_legacy(state: &AppState, app: &mut InteractiveApp) {
    // Sync mode
    app.mode = match (state.view, state.interaction, &state.edit_mode) {
        (ViewState::IssueList, InteractionMode::Normal, _) => AppMode::Normal,
        (ViewState::IssueList, InteractionMode::Search, _) => AppMode::Search,
        (ViewState::IssueDetail, InteractionMode::Normal, _) => AppMode::Detail,
        (ViewState::IssueDetail, InteractionMode::Editing, EditMode::Comment { .. }) => AppMode::Comment,
        (ViewState::IssueDetail, InteractionMode::Editing, _) => AppMode::EditField,
        (ViewState::IssueDetail, InteractionMode::Selecting, _) => AppMode::SelectOption,
        (ViewState::LinkNavigation, _, _) => AppMode::Links,
        (ViewState::ExternalEditor, _, _) => AppMode::ExternalEditor,
        _ => AppMode::Normal,
    };
    
    // Sync navigation
    app.selected_index = state.navigation.issue_index;
    app.selected_link_index = state.navigation.link_index;
    app.option_index = state.navigation.option_index;
    app.selected_issue_id = state.navigation.selected_issue_id.clone();
    
    // Sync edit fields
    match &state.edit_mode {
        EditMode::Title { current_value, .. } => {
            app.edit_field = LegacyEditField::Title;
            app.edit_input = current_value.clone();
        }
        EditMode::Description { current_value, .. } => {
            app.edit_field = LegacyEditField::Description;
            app.edit_input = current_value.clone();
        }
        EditMode::Comment { text, .. } => {
            app.comment_input = text.clone();
        }
        EditMode::Labels { selected_ids, .. } => {
            app.edit_field = LegacyEditField::Labels;
            app.selected_labels = selected_ids.clone();
        }
        _ => {}
    }
    
    // Sync input state
    match app.mode {
        AppMode::Comment => {
            app.comment_input = state.input.content.clone();
            app.comment_cursor_position = state.input.cursor_position;
        }
        AppMode::EditField => {
            app.edit_input = state.input.content.clone();
            app.cursor_position = state.input.cursor_position;
        }
        _ => {}
    }
    
    // Sync other state
    app.search_query = state.search_query.clone();
    app.hide_done_issues = state.hide_completed;
    app.group_by = state.group_by;
    app.error_message = state.error_message.clone();
    app.loading = state.loading;
}