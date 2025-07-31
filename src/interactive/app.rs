use crate::models::Issue;
use crate::client::LinearClient;
use crate::config::get_api_key;
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
    pub selected_issue_id: Option<String>,
    pub edit_field: EditField,
    pub edit_input: String,
    pub edit_field_index: usize,
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
            loading: false,
            error_message: None,
            comment_input: String::new(),
            selected_issue_id: None,
            edit_field: EditField::Title,
            edit_input: String::new(),
            edit_field_index: 0,
        };
        
        app.refresh_issues().await?;
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
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_selection_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection_up(),
            KeyCode::Char('g') => self.toggle_group_by(),
            KeyCode::Char('/') => self.mode = AppMode::Search,
            KeyCode::Char('f') => self.mode = AppMode::Filter,
            KeyCode::Enter => {
                if !self.filtered_issues.is_empty() {
                    self.mode = AppMode::Detail;
                }
            }
            KeyCode::Char('r') => {
                // Refresh issues - handled in main loop
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
            _ => {}
        }
    }

    fn handle_comment_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Detail;
                self.comment_input.clear();
            }
            KeyCode::Enter => {
                // Comment submission will be handled in the main loop
                // because it's async
            }
            KeyCode::Char(c) => {
                self.comment_input.push(c);
            }
            KeyCode::Backspace => {
                self.comment_input.pop();
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
    }

    pub fn get_selected_issue(&self) -> Option<&Issue> {
        self.filtered_issues.get(self.selected_index)
    }

    pub async fn submit_comment(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(issue_id) = &self.selected_issue_id {
            if !self.comment_input.trim().is_empty() {
                self.loading = true;
                match self.client.create_comment(issue_id, &self.comment_input).await {
                    Ok(_) => {
                        self.loading = false;
                        self.comment_input.clear();
                        self.mode = AppMode::Detail;
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
                self.mode = AppMode::Detail;
                self.edit_input.clear();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.edit_field_index > 0 {
                    self.edit_field_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.edit_field_index < 4 { // We have 5 fields (0-4)
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
                    _ => EditField::Title,
                };
                self.edit_input.clear();
                // Pre-fill with current value
                if let Some(issue) = self.get_selected_issue() {
                    self.edit_input = match self.edit_field {
                        EditField::Title => issue.title.clone(),
                        EditField::Description => issue.description.clone().unwrap_or_default(),
                        EditField::Status => issue.state.name.clone(),
                        EditField::Assignee => issue.assignee.as_ref().map(|a| a.name.clone()).unwrap_or_default(),
                        EditField::Priority => match issue.priority {
                            Some(0) => "None".to_string(),
                            Some(1) => "Low".to_string(),
                            Some(2) => "Medium".to_string(),
                            Some(3) => "High".to_string(),
                            Some(4) => "Urgent".to_string(),
                            _ => "None".to_string(),
                        },
                    };
                }
                self.mode = AppMode::EditField;
            }
            _ => {}
        }
    }

    fn handle_edit_field_mode_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Edit;
                self.edit_input.clear();
            }
            KeyCode::Enter => {
                // Submit edit - will be handled in main loop
            }
            KeyCode::Char(c) => {
                self.edit_input.push(c);
            }
            KeyCode::Backspace => {
                self.edit_input.pop();
            }
            _ => {}
        }
    }

    pub async fn submit_edit(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(issue_id) = &self.selected_issue_id {
            if !self.edit_input.trim().is_empty() {
                self.loading = true;
                
                let result = match self.edit_field {
                    EditField::Title => {
                        self.client.update_issue(issue_id, Some(&self.edit_input), None, None, None, None, None).await
                    }
                    EditField::Description => {
                        self.client.update_issue(issue_id, None, Some(&self.edit_input), None, None, None, None).await
                    }
                    _ => {
                        // For now, only support title and description
                        self.loading = false;
                        self.error_message = Some("This field is not yet editable".to_string());
                        return Ok(());
                    }
                };
                
                match result {
                    Ok(_) => {
                        self.loading = false;
                        self.edit_input.clear();
                        self.mode = AppMode::Detail;
                        // Refresh issues to show the update
                        let _ = self.refresh_issues().await;
                    }
                    Err(e) => {
                        self.loading = false;
                        self.error_message = Some(format!("Failed to update: {}", e));
                    }
                }
            }
        }
        Ok(())
    }
}