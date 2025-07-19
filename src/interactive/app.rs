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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Status,
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
}