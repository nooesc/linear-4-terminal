use std::collections::HashSet;
use std::time::Instant;
use crate::models::{Issue, WorkflowState, Comment};
use crate::client::LinearClient;
use crate::config::get_api_key;
use crate::logging::log_error;
use std::error::Error;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Which panel has keyboard focus
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    IssueList,
    DetailPanel,
}

/// Active popup overlay (None = no popup)
#[derive(Debug, Clone, PartialEq)]
pub enum Popup {
    StatusPicker,
    PriorityPicker,
    LabelPicker,
    ProjectPicker,
    AssigneePicker,
    TextInput(TextInputContext),
    Confirmation(ConfirmAction),
    CreateIssue,
    BulkActions,
    Help,
}

/// What the text input popup is being used for
#[derive(Debug, Clone, PartialEq)]
pub enum TextInputContext {
    Comment,
    EditTitle,
    EditDescription,
    Search,
    Filter,
}

/// What a confirmation dialog will do if confirmed
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    ArchiveIssue(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GroupBy {
    Status,
    Project,
}

/// Section within the detail panel
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetailSection {
    Info,
    Description,
    Comments,
}

// ---------------------------------------------------------------------------
// Supporting structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CreateIssueForm {
    pub title: String,
    pub team_id: Option<String>,
    pub status_id: Option<String>,
    pub priority: Option<u8>,
    pub project_id: Option<String>,
    pub label_ids: Vec<String>,
    pub assignee_id: Option<String>,
    pub active_field: usize,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u64,
    pub kind: NotificationKind,
    pub message: String,
    pub created_at: Instant,
    pub dismissed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationKind {
    Success,
    Error,
    Loading,
    Info,
}

// ---------------------------------------------------------------------------
// InteractiveApp
// ---------------------------------------------------------------------------

pub struct InteractiveApp {
    // Layout
    pub focus: Focus,
    pub popup: Option<Popup>,

    // Issue list state
    pub issues: Vec<Issue>,
    pub filtered_issues: Vec<Issue>,
    pub selected_index: usize,
    pub group_by: GroupBy,
    pub hide_done_issues: bool,
    pub multi_selected: HashSet<usize>,

    // Detail panel state
    pub detail_section: DetailSection,
    pub detail_scroll: u16,
    pub comments: Vec<Comment>,
    pub comments_loading: bool,
    pub last_comment_issue_id: Option<String>,

    // Search/filter
    pub search_query: String,
    pub filter_query: String,

    // Text input (reusable for any popup text field)
    pub text_input: String,
    pub text_cursor: usize,

    // Picker state
    pub picker_index: usize,
    pub picker_search: String,
    pub selected_labels: HashSet<String>,

    // Create issue form
    pub create_form: CreateIssueForm,

    // Notifications
    pub notifications: Vec<Notification>,
    pub next_notification_id: u64,

    // Data
    pub client: LinearClient,
    pub workflow_states: Vec<WorkflowState>,
    pub available_labels: Vec<crate::models::issue::Label>,
    pub available_projects: Vec<crate::models::Project>,
    pub team_members: Vec<crate::models::User>,

    // App state
    pub should_quit: bool,
    pub loading: bool,
    pub error_message: Option<String>,

    // External editor
    pub external_editor_pending: bool,
}

impl InteractiveApp {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let api_key = get_api_key()?;
        let client = LinearClient::new(api_key)?;

        let mut app = Self {
            // Layout
            focus: Focus::IssueList,
            popup: None,

            // Issue list
            issues: Vec::new(),
            filtered_issues: Vec::new(),
            selected_index: 0,
            group_by: GroupBy::Status,
            hide_done_issues: false,
            multi_selected: HashSet::new(),

            // Detail panel
            detail_section: DetailSection::Info,
            detail_scroll: 0,
            comments: Vec::new(),
            comments_loading: false,
            last_comment_issue_id: None,

            // Search/filter
            search_query: String::new(),
            filter_query: String::new(),

            // Text input
            text_input: String::new(),
            text_cursor: 0,

            // Picker
            picker_index: 0,
            picker_search: String::new(),
            selected_labels: HashSet::new(),

            // Create form
            create_form: CreateIssueForm::default(),

            // Notifications
            notifications: Vec::new(),
            next_notification_id: 0,

            // Data
            client,
            workflow_states: Vec::new(),
            available_labels: Vec::new(),
            available_projects: Vec::new(),
            team_members: Vec::new(),

            // App state
            should_quit: false,
            loading: true,
            error_message: None,

            // External editor
            external_editor_pending: false,
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

    // -----------------------------------------------------------------------
    // Filters & data
    // -----------------------------------------------------------------------

    pub fn apply_filters(&mut self) {
        self.filtered_issues = self.issues.clone();

        // Apply search filter
        if !self.search_query.is_empty() {
            let query = self.search_query.to_lowercase();
            self.filtered_issues.retain(|issue| {
                issue.title.to_lowercase().contains(&query)
                    || issue.identifier.to_lowercase().contains(&query)
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
                    a.state
                        .name
                        .cmp(&b.state.name)
                        .then(a.priority.cmp(&b.priority).reverse())
                });
            }
            GroupBy::Project => {
                self.filtered_issues.sort_by(|a, b| {
                    let a_project = a.project.as_ref().map(|p| &p.name);
                    let b_project = b.project.as_ref().map(|p| &p.name);
                    a_project
                        .cmp(&b_project)
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

    // -----------------------------------------------------------------------
    // Notifications
    // -----------------------------------------------------------------------

    pub fn notify(&mut self, kind: NotificationKind, message: String) -> u64 {
        let id = self.next_notification_id;
        self.next_notification_id += 1;
        self.notifications.push(Notification {
            id,
            kind,
            message,
            created_at: Instant::now(),
            dismissed: false,
        });
        // Keep max 3
        while self.notifications.len() > 3 {
            self.notifications.remove(0);
        }
        id
    }

    pub fn dismiss_notification(&mut self, id: u64) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.dismissed = true;
        }
    }

    pub fn replace_notification(&mut self, id: u64, kind: NotificationKind, message: String) {
        if let Some(n) = self.notifications.iter_mut().find(|n| n.id == id) {
            n.kind = kind;
            n.message = message;
            n.created_at = Instant::now();
        }
    }

    pub fn tick_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| {
            if n.dismissed {
                return false;
            }
            match n.kind {
                NotificationKind::Success | NotificationKind::Info => {
                    now.duration_since(n.created_at).as_secs() < 5
                }
                NotificationKind::Error | NotificationKind::Loading => true,
            }
        });
    }
}
