use crate::models::{Issue, WorkflowState};
use crate::client::LinearClient;
use crate::config::get_api_key;
use crate::logging::{log_info, log_error, log_debug};
use crossterm::event::KeyCode;
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Filter,
    Detail,
    Comment,
    Edit,
    EditField,
    SelectOption,
    ExternalEditor,
    Links,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Status,
    Project,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditField {
    Title,
    Description,
    Status,
    Assignee,
    Priority,
    Labels,
    Project,
}

pub struct InteractiveApp {
    pub mode: AppMode,
    pub issues: Vec<Issue>,
    pub filtered_issues: Vec<Issue>,
    pub selected_index: usize,
    pub group_by: GroupBy,
    pub search_query: String,
    #[allow(dead_code)]
    pub filter_query: String,
    pub should_quit: bool,
    pub client: LinearClient,
    pub loading: bool,
    pub error_message: Option<String>,
    pub comment_input: String,
    pub comment_cursor_position: usize,
    pub selected_issue_id: Option<String>,
    pub edit_field: EditField,
    pub edit_input: String,
    pub edit_field_index: usize,
    pub workflow_states: Vec<WorkflowState>,
    pub available_labels: Vec<crate::models::issue::Label>,
    pub available_projects: Vec<crate::models::Project>,
    pub selected_labels: Vec<String>, // IDs of selected labels
    pub option_index: usize,
    pub selected_option: Option<String>,
    pub cursor_position: usize,
    pub external_editor_field: Option<EditField>,
    pub current_issue_links: Vec<String>,
    pub selected_link_index: usize,
    pub previous_mode: Option<AppMode>, // Track where we came from for better UX
    pub hide_done_issues: bool, // Toggle to hide completed issues
}

impl InteractiveApp {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = get_api_key()?;
        let client = LinearClient::new(api_key);
        
        let mut app = Self {
            mode: AppMode::Normal,
            issues: Vec::new(),
            filtered_issues: Vec::new(),
            selected_index: 0,
            group_by: GroupBy::Status,
            search_query: String::new(),
            filter_query: String::new(),
            should_quit: false,
            client,
            loading: true,
            error_message: None,
            comment_input: String::new(),
            comment_cursor_position: 0,
            selected_issue_id: None,
            edit_field: EditField::Title,
            edit_input: String::new(),
            edit_field_index: 0,
            workflow_states: Vec::new(),
            available_labels: Vec::new(),
            available_projects: Vec::new(),
            selected_labels: Vec::new(),
            option_index: 0,
            selected_option: None,
            cursor_position: 0,
            external_editor_field: None,
            current_issue_links: Vec::new(),
            selected_link_index: 0,
            previous_mode: None,
            hide_done_issues: false,
        };
        
        // Make all API calls in parallel for faster startup
        let (issues_result, states_result, labels_result, projects_result) = tokio::join!(
            app.client.get_issues(None, Some(100)),
            app.client.get_workflow_states(),
            app.client.get_labels(),
            app.client.get_projects()
        );
        
        // Handle issues result
        match issues_result {
            Ok(issues) => {
                app.issues = issues;
                app.apply_filters();
            }
            Err(e) => {
                app.error_message = Some(format!("Failed to load issues: {}", e));
                return Err(e);
            }
        }
        
        // Handle workflow states result
        match states_result {
            Ok(states) => {
                app.workflow_states = states;
            }
            Err(e) => {
                log_error(&format!("Failed to fetch workflow states: {}", e));
                app.workflow_states = Vec::new();
            }
        }
        
        // Handle labels result
        match labels_result {
            Ok(labels) => {
                app.available_labels = labels;
            }
            Err(e) => {
                log_error(&format!("Failed to fetch labels: {}", e));
                app.available_labels = Vec::new();
            }
        }
        
        // Handle projects result
        match projects_result {
            Ok(projects) => {
                app.available_projects = projects;
            }
            Err(e) => {
                log_error(&format!("Failed to fetch projects: {}", e));
                app.available_projects = Vec::new();
            }
        }
        
        app.loading = false;
        Ok(app)
    }

    pub async fn refresh_issues(&mut self) -> Result<(), Box<dyn Error>> {
        self.loading = true;
        self.error_message = None;
        
        match self.client.get_issues(None, Some(100)).await {
            Ok(issues) => {
                self.issues = issues;
                self.apply_filters();
                self.loading = false;
                Ok(())
            }
            Err(e) => {
                self.loading = false;
                self.error_message = Some(format!("Failed to load issues: {}", e));
                Err(e)
            }
        }
    }

    pub fn apply_filters(&mut self) {
        self.filtered_issues = self.issues.clone();
        
        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.filtered_issues.retain(|issue| {
                issue.title.to_lowercase().contains(&query) ||
                issue.identifier.to_lowercase().contains(&query)
            });
        }
        
        // Filter out done issues if toggle is on
        if self.hide_done_issues {
            self.filtered_issues.retain(|issue| {
                !matches!(issue.state.state_type.as_str(), "completed" | "canceled")
            });
        }
        
        // Apply sorting based on group_by
        match self.group_by {
            GroupBy::Status => {
                self.filtered_issues.sort_by(|a, b| {
                    a.state.name.cmp(&b.state.name)
                        .then(a.priority.cmp(&b.priority).reverse())
                });
            }
            GroupBy::Project => {
                self.filtered_issues.sort_by(|a, b| {
                    let a_project = a.project.as_ref().map(|p| &p.name);
                    let b_project = b.project.as_ref().map(|p| &p.name);
                    a_project.cmp(&b_project)
                        .then(a.state.name.cmp(&b.state.name))
                        .then(a.priority.cmp(&b.priority).reverse())
                });
            }
        }
        
        // Reset selection if needed
        if self.selected_index >= self.filtered_issues.len() && !self.filtered_issues.is_empty() {
            self.selected_index = self.filtered_issues.len() - 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key),
            AppMode::Search => self.handle_search_mode_key(key),
            AppMode::Filter => self.handle_filter_mode_key(key),
            AppMode::Detail => self.handle_detail_mode_key(key),
            AppMode::Comment => self.handle_comment_mode_key(key),
            AppMode::Edit => self.handle_edit_mode_key(key),
            AppMode::EditField => self.handle_edit_field_mode_key(key),
            AppMode::SelectOption => self.handle_select_option_mode_key(key),
            AppMode::ExternalEditor => {}, // External editor is handled in the main loop
            AppMode::Links => self.handle_links_mode_key(key),
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_selection_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection_up(),
            KeyCode::Char('g') => self.toggle_group_by(),
            KeyCode::Char('/') => self.mode = AppMode::Search,
            KeyCode::Char('f') => self.mode = AppMode::Filter,
            KeyCode::Char('o') => {
                // Open current issue in Linear
                if let Some(issue) = self.get_selected_issue() {
                    let _ = self.open_link(&issue.url);
                }
            }
            KeyCode::Char('e') => {
                // Edit current issue
                if let Some(issue) = self.get_selected_issue() {
                    self.selected_issue_id = Some(issue.id.clone());
                    self.edit_field_index = 0;
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::Edit;
                }
            }
            KeyCode::Enter => {
                if !self.filtered_issues.is_empty() {
                    self.mode = AppMode::Detail;
                    // Update current issue links
                    if let Some(issue) = self.get_selected_issue() {
                        self.current_issue_links = super::ui::get_issue_links(issue);
                        self.selected_link_index = 0;
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh issues - handled in main loop
            }
            KeyCode::Char('s') => {
                // Quick edit status
                if let Some(issue) = self.get_selected_issue() {
                    self.selected_issue_id = Some(issue.id.clone());
                    self.edit_field = EditField::Status;
                    self.option_index = 0;
                    self.selected_option = None;
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::SelectOption;
                }
            }
            KeyCode::Char('c') => {
                // Quick comment
                if let Some(issue) = self.get_selected_issue() {
                    self.selected_issue_id = Some(issue.id.clone());
                    self.comment_input.clear();
                    self.comment_cursor_position = 0;
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::Comment;
                }
            }
            KeyCode::Char('l') => {
                // Quick edit labels - go directly to label selection
                log_debug("Handling 'l' key for label edit");
                if let Some(issue) = self.get_selected_issue() {
                    log_debug(&format!("Selected issue: {} - {}", issue.identifier, issue.title));
                    log_debug(&format!("Available labels: {}", self.available_labels.len()));
                    
                    let issue_id = issue.id.clone();
                    let current_label_ids: Vec<String> = issue.labels.nodes.iter()
                        .map(|label| label.id.clone())
                        .collect();
                    
                    self.selected_issue_id = Some(issue_id);
                    self.edit_field = EditField::Labels;
                    self.option_index = 0;
                    self.selected_option = None;
                    self.selected_labels = current_label_ids;
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::SelectOption;
                    
                    log_debug("Successfully set up label selection mode");
                }
            }
            KeyCode::Char('p') => {
                // Quick edit project
                log_debug("Handling 'p' key for project edit");
                if let Some(issue) = self.get_selected_issue() {
                    log_debug(&format!("Selected issue: {} - {}", issue.identifier, issue.title));
                    log_debug(&format!("Current project: {:?}", issue.project.as_ref().map(|p| &p.name)));
                    log_debug(&format!("Available projects: {}", self.available_projects.len()));
                    
                    self.selected_issue_id = Some(issue.id.clone());
                    self.edit_field = EditField::Project;
                    self.option_index = 0; // Always start at "None" option
                    self.selected_option = None;
                    self.previous_mode = Some(self.mode);
                    self.mode = AppMode::SelectOption;
                    
                    log_debug("Successfully set up project selection mode");
                } else {
                    log_error("No issue selected when pressing 'p'");
                }
            }
            KeyCode::Char('d') => {
                // Toggle hiding done issues
                self.hide_done_issues = !self.hide_done_issues;
                self.apply_filters();
            }
            _ => {}
        }
    }

    fn handle_search_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_query.clear();
                self.apply_filters();
            }
            KeyCode::Enter => {
                self.mode = AppMode::Normal;
                self.apply_filters();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filters();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filters();
            }
            _ => {}
        }
    }

    fn handle_filter_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
    }

    fn handle_detail_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('c') => {
                if let Some(issue) = self.get_selected_issue() {
                    self.selected_issue_id = Some(issue.id.clone());
                    self.comment_input.clear();
                    self.comment_cursor_position = 0;
                    self.mode = AppMode::Comment;
                }
            }
            KeyCode::Char('e') => {
                if let Some(issue) = self.get_selected_issue() {
                    self.selected_issue_id = Some(issue.id.clone());
                    self.edit_field_index = 0;
                    self.mode = AppMode::Edit;
                }
            }
            KeyCode::Char('o') => {
                // Open Linear issue URL
                if !self.current_issue_links.is_empty() {
                    let _ = self.open_link(&self.current_issue_links[0]);
                }
            }
            KeyCode::Char('l') => {
                // Enter links navigation mode
                if self.current_issue_links.len() > 1 {
                    self.selected_link_index = 0;
                    self.mode = AppMode::Links;
                }
            }
            KeyCode::Char(c) if c.is_digit(10) => {
                // Open numbered link
                let index = c.to_digit(10).unwrap() as usize;
                if index < self.current_issue_links.len() {
                    let _ = self.open_link(&self.current_issue_links[index]);
                }
            }
            _ => {}
        }
    }

    fn handle_comment_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                // Return to previous mode when canceling
                self.mode = self.previous_mode.take().unwrap_or(AppMode::Detail);
                self.comment_input.clear();
                self.comment_cursor_position = 0;
            }
            KeyCode::Enter => {
                // Comment submission will be handled in the main loop
                // because it's async
            }
            KeyCode::Char(c) => {
                self.comment_input.insert(self.comment_cursor_position, c);
                self.comment_cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.comment_cursor_position > 0 {
                    self.comment_input.remove(self.comment_cursor_position - 1);
                    self.comment_cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if self.comment_cursor_position < self.comment_input.len() {
                    self.comment_input.remove(self.comment_cursor_position);
                }
            }
            KeyCode::Left => {
                if self.comment_cursor_position > 0 {
                    self.comment_cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.comment_cursor_position < self.comment_input.len() {
                    self.comment_cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.comment_cursor_position = 0;
            }
            KeyCode::End => {
                self.comment_cursor_position = self.comment_input.len();
            }
            _ => {}
        }
    }

    fn move_selection_down(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_issues.len();
        }
    }

    fn move_selection_up(&mut self) {
        if !self.filtered_issues.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_issues.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    fn toggle_group_by(&mut self) {
        self.group_by = match self.group_by {
            GroupBy::Status => GroupBy::Project,
            GroupBy::Project => GroupBy::Status,
        };
        // Re-apply filters to trigger re-sorting
        self.apply_filters();
    }

    pub fn get_selected_issue(&self) -> Option<&Issue> {
        self.filtered_issues.get(self.selected_index)
    }
    
    pub fn get_issue_by_id(&self, id: &str) -> Option<&Issue> {
        self.issues.iter().find(|i| i.id == id)
    }
    
    pub fn open_link(&self, url: &str) -> Result<(), Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        let cmd = "open";
        #[cfg(target_os = "windows")]
        let cmd = "start";
        #[cfg(target_os = "linux")]
        let cmd = "xdg-open";
        
        std::process::Command::new(cmd)
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open link: {}", e))?;
        
        Ok(())
    }

    pub async fn submit_comment(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(issue_id) = &self.selected_issue_id {
            if !self.comment_input.trim().is_empty() {
                self.loading = true;
                match self.client.create_comment(issue_id, &self.comment_input).await {
                    Ok(_) => {
                        self.loading = false;
                        self.comment_input.clear();
                        // Return to previous mode or default to Detail
                        self.mode = self.previous_mode.take().unwrap_or(AppMode::Detail);
                        Ok(())
                    }
                    Err(e) => {
                        self.loading = false;
                        self.error_message = Some(format!("Failed to add comment: {}", e));
                        Err(e)
                    }
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn handle_edit_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Return to previous mode when canceling
                self.mode = self.previous_mode.take().unwrap_or(AppMode::Detail);
                self.edit_input.clear();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.edit_field_index > 0 {
                    self.edit_field_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.edit_field_index < 6 { // We have 7 fields (0-6)
                    self.edit_field_index += 1;
                }
            }
            KeyCode::Enter => {
                self.edit_field = match self.edit_field_index {
                    0 => EditField::Title,
                    1 => EditField::Description,
                    2 => EditField::Status,
                    3 => EditField::Assignee,
                    4 => EditField::Priority,
                    5 => EditField::Labels,
                    6 => EditField::Project,
                    _ => EditField::Title,
                };
                self.edit_input.clear();
                
                // For status, priority, labels, and project, show selection mode
                match self.edit_field {
                    EditField::Status | EditField::Priority | EditField::Labels | EditField::Project => {
                        self.option_index = 0;
                        self.selected_option = None;
                        
                        // For labels, populate selected_labels with current issue's labels
                        if self.edit_field == EditField::Labels {
                            if let Some(issue) = self.get_selected_issue() {
                                self.selected_labels = issue.labels.nodes.iter()
                                    .map(|label| label.id.clone())
                                    .collect();
                            } else {
                                self.selected_labels.clear();
                            }
                        } else if self.edit_field == EditField::Project {
                            // For project, set selected_option to current project ID
                            if let Some(issue) = self.get_selected_issue() {
                                self.selected_option = issue.project.as_ref().map(|p| p.id.clone());
                            }
                        }
                        
                        self.mode = AppMode::SelectOption;
                    }
                    _ => {
                        // Pre-fill with current value for text fields
                        if let Some(issue) = self.get_selected_issue() {
                            self.edit_input = match self.edit_field {
                                EditField::Title => issue.title.clone(),
                                EditField::Description => {
                                    // For description, provide a template if empty
                                    let desc = issue.description.clone().unwrap_or_default();
                                    if desc.trim().is_empty() {
                                        "".to_string() // Start with empty for new descriptions
                                    } else {
                                        desc
                                    }
                                },
                                EditField::Assignee => issue.assignee.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                                _ => String::new(),
                            };
                        }
                        self.cursor_position = self.edit_input.len();
                        self.mode = AppMode::EditField;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_edit_field_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                // Go back to Edit menu, but preserve previous_mode
                self.mode = AppMode::Edit;
                self.edit_input.clear();
                self.cursor_position = 0;
            }
            KeyCode::Enter => {
                // Submit edit - will be handled in main loop
            }
            KeyCode::Char('\x05') => {
                // Ctrl+E - launch external editor for description
                if self.edit_field == EditField::Description {
                    self.prepare_external_editor();
                }
            }
            KeyCode::Char(c) => {
                self.edit_input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.edit_input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.edit_input.len() {
                    self.edit_input.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.edit_input.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.edit_input.len();
            }
            _ => {}
        }
    }

    fn handle_select_option_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                // If we have a previous mode, return to it instead of Edit
                self.mode = self.previous_mode.take().unwrap_or(AppMode::Edit);
                self.option_index = 0;
                self.selected_option = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.option_index > 0 {
                    self.option_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max_index = match self.edit_field {
                    EditField::Status => self.workflow_states.len().saturating_sub(1),
                    EditField::Priority => 4, // 0-4 for None, Low, Medium, High, Urgent
                    EditField::Labels => {
                        // If no labels, max index is 0 (can't navigate)
                        // Otherwise, max index is len - 1
                        if self.available_labels.is_empty() {
                            0
                        } else {
                            self.available_labels.len() - 1
                        }
                    },
                    EditField::Project => {
                        // Include "None" option, so total is projects.len() + 1
                        // But max index is projects.len() (since we start from 0)
                        // If no projects, we only have "None" option, so max index is 0
                        if self.available_projects.is_empty() {
                            0
                        } else {
                            self.available_projects.len()
                        }
                    },
                    _ => 0,
                };
                if self.option_index < max_index {
                    self.option_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                match self.edit_field {
                    EditField::Status => {
                        if let Some(state) = self.workflow_states.get(self.option_index) {
                            self.selected_option = Some(state.id.clone());
                            if key == KeyCode::Enter {
                                self.loading = true;
                            }
                        }
                    }
                    EditField::Priority => {
                        self.selected_option = Some(self.option_index.to_string());
                        if key == KeyCode::Enter {
                            self.loading = true;
                        }
                    }
                    EditField::Labels => {
                        // Toggle label selection with space bar
                        log_debug(&format!("Label selection: option_index={}, available_labels={}", self.option_index, self.available_labels.len()));
                        if !self.available_labels.is_empty() {
                            if let Some(label) = self.available_labels.get(self.option_index) {
                                let label_id = label.id.clone();
                                if let Some(pos) = self.selected_labels.iter().position(|id| id == &label_id) {
                                    self.selected_labels.remove(pos);
                                } else {
                                    self.selected_labels.push(label_id);
                                }
                                // Don't close menu on space, only on Enter
                                if key == KeyCode::Char(' ') {
                                    return;
                                }
                                if key == KeyCode::Enter {
                                    self.loading = true;
                                }
                            }
                        } else if key == KeyCode::Enter {
                            // No labels available, just close the dialog
                            self.mode = self.previous_mode.unwrap_or(AppMode::Normal);
                        }
                    }
                    EditField::Project => {
                        log_debug(&format!("Project selection: option_index={}, available_projects={}", self.option_index, self.available_projects.len()));
                        if self.option_index == 0 {
                            // "None" option selected
                            self.selected_option = Some("none".to_string());
                        } else if self.option_index > 0 && self.option_index <= self.available_projects.len() {
                            // Make sure we're within bounds
                            if let Some(project) = self.available_projects.get(self.option_index - 1) {
                                self.selected_option = Some(project.id.clone());
                            }
                        }
                        if key == KeyCode::Enter {
                            self.loading = true;
                        }
                    }
                    _ => {}
                }
                // Submit will be handled in main loop
            }
            _ => {}
        }
    }
    
    fn handle_links_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Detail;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_link_index > 0 {
                    self.selected_link_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_link_index < self.current_issue_links.len().saturating_sub(1) {
                    self.selected_link_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char('o') => {
                if let Some(link) = self.current_issue_links.get(self.selected_link_index) {
                    let _ = self.open_link(link);
                }
            }
            _ => {}
        }
    }

    pub fn prepare_external_editor(&mut self) -> Option<String> {
        if self.edit_field == EditField::Description {
            // If edit_input is empty, populate it with current description
            if self.edit_input.is_empty() {
                if let Some(issue) = self.get_selected_issue() {
                    self.edit_input = issue.description.clone().unwrap_or_default();
                }
            }
            
            self.external_editor_field = Some(self.edit_field);
            self.mode = AppMode::ExternalEditor;
            Some(self.edit_input.clone())
        } else {
            None
        }
    }
    
    pub fn handle_external_editor_result(&mut self, content: Option<String>) {
        if let Some(new_content) = content {
            self.edit_input = new_content;
        }
        self.mode = AppMode::EditField;
        self.external_editor_field = None;
    }

    pub async fn submit_edit(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(issue_id) = &self.selected_issue_id {
            self.loading = true;
            
            let result = match self.edit_field {
                EditField::Title => {
                    if !self.edit_input.trim().is_empty() {
                        self.client.update_issue(issue_id, Some(&self.edit_input), None, None, None, None, None).await
                    } else {
                        self.loading = false;
                        return Ok(());
                    }
                }
                EditField::Description => {
                    self.client.update_issue(issue_id, None, Some(&self.edit_input), None, None, None, None).await
                }
                EditField::Status => {
                    if let Some(state_id) = &self.selected_option {
                        self.client.update_issue(issue_id, None, None, Some(state_id), None, None, None).await
                    } else {
                        self.loading = false;
                        return Ok(());
                    }
                }
                EditField::Priority => {
                    if let Some(priority_str) = &self.selected_option {
                        if let Ok(priority) = priority_str.parse::<u8>() {
                            self.client.update_issue(issue_id, None, None, None, Some(priority), None, None).await
                        } else {
                            self.loading = false;
                            return Ok(());
                        }
                    } else {
                        self.loading = false;
                        return Ok(());
                    }
                }
                EditField::Assignee => {
                    // For now, assignee still uses text input
                    self.loading = false;
                    self.error_message = Some("Assignee field is not yet editable".to_string());
                    return Ok(());
                }
                EditField::Labels => {
                    let label_ids: Vec<&str> = self.selected_labels.iter()
                        .map(|s| s.as_str())
                        .collect();
                    self.client.update_issue(issue_id, None, None, None, None, None, Some(label_ids)).await
                }
                EditField::Project => {
                    if let Some(project_option) = &self.selected_option {
                        if project_option == "none" {
                            // Remove project by setting to null
                            self.client.update_issue_with_project(issue_id, None, None, None, None, None, None, Some(None)).await
                        } else {
                            // Set to selected project
                            self.client.update_issue_with_project(issue_id, None, None, None, None, None, None, Some(Some(project_option.as_str()))).await
                        }
                    } else {
                        self.loading = false;
                        return Ok(());
                    }
                }
            };
            
            match result {
                Ok(_) => {
                    self.loading = false;
                    self.edit_input.clear();
                    self.selected_option = None;
                    self.selected_labels.clear();
                    // Return to previous mode or default to Normal
                    self.mode = self.previous_mode.take().unwrap_or(AppMode::Normal);
                    // Refresh issues to show the update
                    let _ = self.refresh_issues().await;
                }
                Err(e) => {
                    self.loading = false;
                    self.error_message = Some(format!("Failed to update: {}", e));
                }
            }
        }
        Ok(())
    }
}