use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::config::get_api_key;
use crate::interactive::app::{
    ConfirmAction, CreateIssueForm, Focus, GroupBy, InteractiveApp, NotificationKind, Popup,
    TextInputContext,
};
use crate::interactive::keys::{self, Action};
use super::event::{Event, EventHandler};

pub async fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Check API key
    let api_key = get_api_key()?;
    if api_key.is_empty() {
        eprintln!("No API key found. Run: linear auth <your-api-key>");
        return Ok(());
    }

    // 2. Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Create app
    let mut app = InteractiveApp::new().await?;
    let events = EventHandler::new(100); // 100ms tick rate

    // Track which issue we last fetched comments for
    let mut last_detail_issue_id: Option<String> = None;

    // Main loop
    loop {
        // Tick notifications
        app.tick_notifications();

        // Draw UI
        terminal.draw(|f| super::ui::draw(f, &app))?;

        // Fetch comments if selected issue changed
        if let Some(issue) = app.get_selected_issue() {
            let issue_id = issue.id.clone();
            if last_detail_issue_id.as_ref() != Some(&issue_id) {
                last_detail_issue_id = Some(issue_id.clone());
                app.comments_loading = true;
                app.comments.clear();
                match app.client.get_comments(&issue_id).await {
                    Ok(comments) => {
                        app.comments = comments;
                        app.comments_loading = false;
                    }
                    Err(_) => {
                        app.comments_loading = false;
                    }
                }
            }
        }

        // Handle events
        match events.recv()? {
            Event::Key(key) => {
                let action = keys::map_key(key, &app.focus, &app.popup);
                handle_action(&mut app, action).await;
            }
            Event::Tick => {}
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Central action dispatcher
// ---------------------------------------------------------------------------

async fn handle_action(app: &mut InteractiveApp, action: Action) {
    match action {
        // Navigation
        Action::MoveUp => {
            match app.focus {
                Focus::TeamList => {
                    if app.team_index > 0 {
                        app.team_index -= 1;
                    }
                }
                Focus::ProjectList => {
                    if app.project_index > 0 {
                        app.project_index -= 1;
                    }
                }
                Focus::IssueList => {
                    if app.selected_index > 0 {
                        app.selected_index -= 1;
                        app.detail_scroll = 0;
                    }
                }
                Focus::DetailPanel => {
                    app.detail_scroll = app.detail_scroll.saturating_sub(1);
                }
            }
        }
        Action::MoveDown => {
            match app.focus {
                Focus::TeamList => {
                    if app.team_index < app.teams.len().saturating_sub(1) {
                        app.team_index += 1;
                    }
                }
                Focus::ProjectList => {
                    // +1 for "All" entry
                    let max = app.available_projects.len(); // 0="All", so max index = len
                    if app.project_index < max {
                        app.project_index += 1;
                    }
                }
                Focus::IssueList => {
                    if app.selected_index < app.filtered_issues.len().saturating_sub(1) {
                        app.selected_index += 1;
                        app.detail_scroll = 0;
                    }
                }
                Focus::DetailPanel => {
                    app.detail_scroll += 1;
                }
            }
        }
        Action::ScrollUp => {
            app.detail_scroll = app.detail_scroll.saturating_sub(1);
        }
        Action::ScrollDown => {
            app.detail_scroll += 1;
        }

        // Focus
        Action::SwitchPanel => {
            app.focus = match app.focus {
                Focus::TeamList => Focus::ProjectList,
                Focus::ProjectList => Focus::IssueList,
                Focus::IssueList => Focus::DetailPanel,
                Focus::DetailPanel => Focus::TeamList,
            };
        }
        Action::FocusList => {
            // Shift-Tab: go backwards in focus cycle
            app.focus = match app.focus {
                Focus::TeamList => Focus::DetailPanel,
                Focus::ProjectList => Focus::TeamList,
                Focus::IssueList => Focus::ProjectList,
                Focus::DetailPanel => Focus::IssueList,
            };
        }

        // Popups
        Action::ChangeStatus => {
            app.picker_index = 0;
            app.popup = Some(Popup::StatusPicker);
        }
        Action::AddComment => {
            app.text_input.clear();
            app.text_cursor = 0;
            app.popup = Some(Popup::TextInput(TextInputContext::Comment));
        }
        Action::ChangeLabels => {
            app.picker_index = 0;
            // Pre-select current labels
            if let Some(issue) = app.get_selected_issue() {
                app.selected_labels = issue
                    .labels
                    .nodes
                    .iter()
                    .map(|l| l.id.clone())
                    .collect();
            }
            app.popup = Some(Popup::LabelPicker);
        }
        Action::ChangeProject => {
            app.picker_index = 0;
            app.popup = Some(Popup::ProjectPicker);
        }
        Action::ChangeAssignee => {
            app.picker_index = 0;
            app.popup = Some(Popup::AssigneePicker);
        }
        Action::NewIssue => {
            app.create_form = CreateIssueForm::default();
            app.popup = Some(Popup::CreateIssue);
        }
        Action::Search => {
            app.text_input = app.search_query.clone();
            app.text_cursor = app.text_input.len();
            app.popup = Some(Popup::TextInput(TextInputContext::Search));
        }
        Action::Filter => {
            app.text_input = app.filter_query.clone();
            app.text_cursor = app.text_input.len();
            app.popup = Some(Popup::TextInput(TextInputContext::Filter));
        }
        Action::Help => {
            app.popup = Some(Popup::Help);
        }
        Action::EditFull => {
            // Open title edit as default
            if let Some(issue) = app.get_selected_issue() {
                app.text_input = issue.title.clone();
                app.text_cursor = app.text_input.len();
                app.popup = Some(Popup::TextInput(TextInputContext::EditTitle));
            }
        }

        // Multi-select
        Action::ToggleSelect => {
            let idx = app.selected_index;
            if app.multi_selected.contains(&idx) {
                app.multi_selected.remove(&idx);
            } else {
                app.multi_selected.insert(idx);
            }
        }
        Action::ClearSelection => {
            app.multi_selected.clear();
        }
        Action::BulkAction => {
            if !app.multi_selected.is_empty() {
                app.picker_index = 0;
                app.popup = Some(Popup::BulkActions);
            }
        }

        // Text input actions
        Action::TypeChar(c) => handle_type_char(app, c),
        Action::Backspace => handle_backspace(app),
        Action::Delete => handle_delete(app),
        Action::MoveCursorLeft => handle_cursor_left(app),
        Action::MoveCursorRight => handle_cursor_right(app),
        Action::CursorHome => handle_cursor_home(app),
        Action::CursorEnd => handle_cursor_end(app),
        Action::NextField => handle_next_field(app),
        Action::PrevField => handle_prev_field(app),

        // Confirm/Cancel
        Action::Confirm => handle_confirm(app).await,
        Action::Cancel => {
            app.popup = None;
        }

        // Picker
        Action::PickerUp => {
            app.picker_index = app.picker_index.saturating_sub(1);
        }
        Action::PickerDown => {
            let max = match &app.popup {
                Some(Popup::StatusPicker) => app.workflow_states.len().saturating_sub(1),
                Some(Popup::PriorityPicker) => 4,
                Some(Popup::LabelPicker) => app.available_labels.len().saturating_sub(1),
                Some(Popup::ProjectPicker) => app.available_projects.len(), // includes "None" at 0
                Some(Popup::AssigneePicker) => app.team_members.len(),      // includes "Unassign" at 0
                Some(Popup::BulkActions) => 5,
                _ => 0,
            };
            if app.picker_index < max {
                app.picker_index += 1;
            }
        }
        Action::PickerConfirm => handle_picker_confirm(app).await,
        Action::PickerCancel => {
            app.popup = None;
            app.bulk_mode = false;
        }
        Action::PickerToggle => handle_picker_toggle(app),

        // Other
        Action::OpenInBrowser => {
            if let Some(issue) = app.get_selected_issue() {
                let url = issue.url.clone();
                let _ = app.open_link(&url);
            }
        }
        Action::ToggleDone => {
            app.hide_done_issues = !app.hide_done_issues;
            app.apply_filters();
        }
        Action::GroupBy => {
            app.group_by = match app.group_by {
                GroupBy::Status => GroupBy::Project,
                GroupBy::Project => GroupBy::Status,
            };
            app.apply_filters();
        }
        Action::Refresh => {
            let nid = app.notify(NotificationKind::Loading, "Refreshing issues...".into());
            match app.refresh_issues().await {
                Ok(_) => app.replace_notification(
                    nid,
                    NotificationKind::Success,
                    "Issues refreshed".into(),
                ),
                Err(e) => app.replace_notification(
                    nid,
                    NotificationKind::Error,
                    format!("Refresh failed: {}", e),
                ),
            }
        }
        Action::Quit => {
            app.should_quit = true;
        }
        Action::DismissNotification => {
            if let Some(n) = app
                .notifications
                .iter_mut()
                .find(|n| n.kind == NotificationKind::Error && !n.dismissed)
            {
                n.dismissed = true;
            }
        }
        Action::SelectTeam => {
            if app.team_index < app.teams.len() {
                let was_same = app.active_team == Some(app.team_index);
                if was_same {
                    // Deselect: show all teams
                    app.active_team = None;
                } else {
                    app.active_team = Some(app.team_index);
                }
                // Reset project selection
                app.active_project = None;
                app.project_index = 0;
                // Re-fetch issues
                let team_name = app
                    .teams
                    .get(app.team_index)
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                let msg = if was_same {
                    "Showing all teams".to_string()
                } else {
                    format!("Filtering by team: {}", team_name)
                };
                let nid = app.notify(NotificationKind::Loading, msg.clone());
                match app.refresh_issues().await {
                    Ok(_) => app.replace_notification(nid, NotificationKind::Success, msg),
                    Err(e) => app.replace_notification(
                        nid,
                        NotificationKind::Error,
                        format!("Failed: {}", e),
                    ),
                }
            }
        }
        Action::SelectProject => {
            let max_idx = app.available_projects.len(); // 0=All, 1..=len=projects
            if app.project_index <= max_idx {
                if app.project_index == 0 {
                    app.active_project = None; // "All"
                } else {
                    app.active_project = Some(app.project_index);
                }
                let msg = if app.project_index == 0 {
                    "Showing all projects".to_string()
                } else {
                    let name = app
                        .available_projects
                        .get(app.project_index - 1)
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    format!("Filtering by project: {}", name)
                };
                let nid = app.notify(NotificationKind::Loading, msg.clone());
                match app.refresh_issues().await {
                    Ok(_) => app.replace_notification(nid, NotificationKind::Success, msg),
                    Err(e) => app.replace_notification(
                        nid,
                        NotificationKind::Error,
                        format!("Failed: {}", e),
                    ),
                }
            }
        }
        Action::ExternalEditor => {
            // TODO: external editor support
        }
        Action::None => {}
    }
}

// ---------------------------------------------------------------------------
// Text input helpers
// ---------------------------------------------------------------------------

fn handle_type_char(app: &mut InteractiveApp, c: char) {
    match &app.popup {
        Some(Popup::CreateIssue) => {
            // Only type into title field (active_field == 0)
            if app.create_form.active_field == 0 {
                app.create_form.title.insert(app.text_cursor, c);
                app.text_cursor += 1;
            }
        }
        Some(Popup::TextInput(_)) => {
            app.text_input.insert(app.text_cursor, c);
            app.text_cursor += 1;
        }
        _ => {}
    }
}

fn handle_backspace(app: &mut InteractiveApp) {
    match &app.popup {
        Some(Popup::CreateIssue) => {
            if app.create_form.active_field == 0 && app.text_cursor > 0 {
                app.text_cursor -= 1;
                app.create_form.title.remove(app.text_cursor);
            }
        }
        Some(Popup::TextInput(_)) => {
            if app.text_cursor > 0 {
                app.text_cursor -= 1;
                app.text_input.remove(app.text_cursor);
            }
        }
        _ => {}
    }
}

fn handle_delete(app: &mut InteractiveApp) {
    match &app.popup {
        Some(Popup::TextInput(_)) => {
            if app.text_cursor < app.text_input.len() {
                app.text_input.remove(app.text_cursor);
            }
        }
        _ => {}
    }
}

fn handle_cursor_left(app: &mut InteractiveApp) {
    app.text_cursor = app.text_cursor.saturating_sub(1);
}

fn handle_cursor_right(app: &mut InteractiveApp) {
    let max = match &app.popup {
        Some(Popup::CreateIssue) => app.create_form.title.len(),
        Some(Popup::TextInput(_)) => app.text_input.len(),
        _ => 0,
    };
    if app.text_cursor < max {
        app.text_cursor += 1;
    }
}

fn handle_cursor_home(app: &mut InteractiveApp) {
    app.text_cursor = 0;
}

fn handle_cursor_end(app: &mut InteractiveApp) {
    app.text_cursor = match &app.popup {
        Some(Popup::CreateIssue) => app.create_form.title.len(),
        Some(Popup::TextInput(_)) => app.text_input.len(),
        _ => 0,
    };
}

fn handle_next_field(app: &mut InteractiveApp) {
    if matches!(app.popup, Some(Popup::CreateIssue)) {
        if app.create_form.active_field < 6 {
            app.create_form.active_field += 1;
        }
    }
}

fn handle_prev_field(app: &mut InteractiveApp) {
    if matches!(app.popup, Some(Popup::CreateIssue)) {
        if app.create_form.active_field > 0 {
            app.create_form.active_field -= 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Confirm handler — text input and confirmation submissions
// ---------------------------------------------------------------------------

async fn handle_confirm(app: &mut InteractiveApp) {
    let popup = app.popup.clone();
    match popup {
        Some(Popup::TextInput(TextInputContext::Comment)) => {
            if let Some(issue) = app.get_selected_issue() {
                let issue_id = issue.id.clone();
                let body = app.text_input.clone();
                if !body.trim().is_empty() {
                    app.popup = None;
                    let nid = app.notify(NotificationKind::Loading, "Adding comment...".into());
                    match app.client.create_comment(&issue_id, &body).await {
                        Ok(_) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Success,
                                "Comment added".into(),
                            );
                            // Force refetch of comments
                            app.last_comment_issue_id = None;
                        }
                        Err(e) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Error,
                                format!("Failed: {}", e),
                            );
                        }
                    }
                }
            }
        }
        Some(Popup::TextInput(TextInputContext::Search)) => {
            app.search_query = app.text_input.clone();
            app.apply_filters();
            app.popup = None;
        }
        Some(Popup::TextInput(TextInputContext::Filter)) => {
            app.filter_query = app.text_input.clone();
            // TODO: apply filter parser
            app.popup = None;
        }
        Some(Popup::TextInput(TextInputContext::EditTitle)) => {
            if let Some(issue) = app.get_selected_issue() {
                let issue_id = issue.id.clone();
                let title = app.text_input.clone();
                if !title.trim().is_empty() {
                    app.popup = None;
                    let nid =
                        app.notify(NotificationKind::Loading, "Updating title...".into());
                    match app
                        .client
                        .update_issue(&issue_id, Some(&title), None, None, None, None, None)
                        .await
                    {
                        Ok(_) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Success,
                                "Title updated".into(),
                            );
                            let _ = app.refresh_issues().await;
                        }
                        Err(e) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Error,
                                format!("Failed: {}", e),
                            );
                        }
                    }
                }
            }
        }
        Some(Popup::TextInput(TextInputContext::EditDescription)) => {
            if let Some(issue) = app.get_selected_issue() {
                let issue_id = issue.id.clone();
                let desc = app.text_input.clone();
                app.popup = None;
                let nid =
                    app.notify(NotificationKind::Loading, "Updating description...".into());
                match app
                    .client
                    .update_issue(&issue_id, None, Some(&desc), None, None, None, None)
                    .await
                {
                    Ok(_) => {
                        app.replace_notification(
                            nid,
                            NotificationKind::Success,
                            "Description updated".into(),
                        );
                        let _ = app.refresh_issues().await;
                    }
                    Err(e) => {
                        app.replace_notification(
                            nid,
                            NotificationKind::Error,
                            format!("Failed: {}", e),
                        );
                    }
                }
            }
        }
        Some(Popup::Confirmation(ConfirmAction::ArchiveIssue(issue_id))) => {
            app.popup = None;
            let nid = app.notify(NotificationKind::Loading, "Archiving issue...".into());
            match app.client.archive_issue(&issue_id).await {
                Ok(_) => {
                    app.replace_notification(
                        nid,
                        NotificationKind::Success,
                        "Issue archived".into(),
                    );
                    let _ = app.refresh_issues().await;
                }
                Err(e) => {
                    app.replace_notification(
                        nid,
                        NotificationKind::Error,
                        format!("Failed: {}", e),
                    );
                }
            }
        }
        Some(Popup::CreateIssue) => {
            // Submit issue creation
            if !app.create_form.title.trim().is_empty() {
                let title = app.create_form.title.clone();
                // Use team_id from form, or extract from first loaded issue
                let team_id = app
                    .create_form
                    .team_id
                    .clone()
                    .or_else(|| app.issues.first().map(|i| i.team.id.clone()));

                if let Some(team_id) = team_id {
                    app.popup = None;
                    let nid =
                        app.notify(NotificationKind::Loading, "Creating issue...".into());
                    let label_ids_owned = app.create_form.label_ids.clone();
                    let label_refs: Vec<&str> =
                        label_ids_owned.iter().map(|s| s.as_str()).collect();
                    let labels_arg = if label_refs.is_empty() {
                        None
                    } else {
                        Some(label_refs)
                    };
                    match app
                        .client
                        .create_issue(
                            &title,
                            None,
                            &team_id,
                            app.create_form.priority,
                            app.create_form.assignee_id.as_deref(),
                            labels_arg,
                        )
                        .await
                    {
                        Ok(_) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Success,
                                format!("Created: {}", title),
                            );
                            let _ = app.refresh_issues().await;
                        }
                        Err(e) => {
                            app.replace_notification(
                                nid,
                                NotificationKind::Error,
                                format!("Failed: {}", e),
                            );
                        }
                    }
                } else {
                    app.notify(
                        NotificationKind::Error,
                        "No team available — cannot create issue".into(),
                    );
                }
            }
        }
        _ => {
            app.popup = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Picker confirm handler
// ---------------------------------------------------------------------------

/// Get target issue IDs: multi-selected (bulk mode) or just the selected issue
fn get_target_ids(app: &InteractiveApp) -> Vec<String> {
    if app.bulk_mode && !app.multi_selected.is_empty() {
        app.get_multi_selected_issue_ids()
    } else if let Some(issue) = app.get_selected_issue() {
        vec![issue.id.clone()]
    } else {
        vec![]
    }
}

/// Finish a bulk/single update: show result notification, clean up bulk state, refresh
async fn finish_update(app: &mut InteractiveApp, nid: u64, success: usize, total: usize, action: &str, last_err: &str) {
    if success == total {
        let msg = if total > 1 {
            format!("{} ({} issues)", action, total)
        } else {
            action.to_string()
        };
        app.replace_notification(nid, NotificationKind::Success, msg);
    } else {
        let msg = if total > 1 {
            format!("Failed {}/{}: {}", total - success, total, last_err)
        } else {
            format!("Failed: {}", last_err)
        };
        app.replace_notification(nid, NotificationKind::Error, msg);
    }
    if app.bulk_mode {
        app.multi_selected.clear();
        app.bulk_mode = false;
    }
    let _ = app.refresh_issues().await;
}

async fn handle_picker_confirm(app: &mut InteractiveApp) {
    let popup = app.popup.clone();
    match popup {
        Some(Popup::StatusPicker) => {
            if let Some(state) = app.workflow_states.get(app.picker_index) {
                let state_id = state.id.clone();
                let state_name = state.name.clone();
                let ids = get_target_ids(app);
                if !ids.is_empty() {
                    app.popup = None;
                    let action = format!("Status -> {}", state_name);
                    let nid = app.notify(NotificationKind::Loading, format!("{}...", action));
                    let mut ok = 0;
                    let mut err = String::new();
                    for id in &ids {
                        match app.client.update_issue(id, None, None, Some(&state_id), None, None, None).await {
                            Ok(_) => ok += 1,
                            Err(e) => err = e.to_string(),
                        }
                    }
                    finish_update(app, nid, ok, ids.len(), &action, &err).await;
                }
            }
        }
        Some(Popup::PriorityPicker) => {
            let priority = app.picker_index as u8;
            let names = ["None", "Low", "Medium", "High", "Urgent"];
            let name = names.get(app.picker_index).unwrap_or(&"Unknown").to_string();
            let ids = get_target_ids(app);
            if !ids.is_empty() {
                app.popup = None;
                let action = format!("Priority -> {}", name);
                let nid = app.notify(NotificationKind::Loading, format!("{}...", action));
                let mut ok = 0;
                let mut err = String::new();
                for id in &ids {
                    match app.client.update_issue(id, None, None, None, Some(priority), None, None).await {
                        Ok(_) => ok += 1,
                        Err(e) => err = e.to_string(),
                    }
                }
                finish_update(app, nid, ok, ids.len(), &action, &err).await;
            }
        }
        Some(Popup::LabelPicker) => {
            let label_ids: Vec<String> = app.selected_labels.iter().cloned().collect();
            let ids = get_target_ids(app);
            if !ids.is_empty() {
                app.popup = None;
                let action = "Labels updated";
                let nid = app.notify(NotificationKind::Loading, "Updating labels...".into());
                let mut ok = 0;
                let mut err = String::new();
                for id in &ids {
                    let refs: Vec<&str> = label_ids.iter().map(|s| s.as_str()).collect();
                    match app.client.update_issue(id, None, None, None, None, None, Some(refs)).await {
                        Ok(_) => ok += 1,
                        Err(e) => err = e.to_string(),
                    }
                }
                finish_update(app, nid, ok, ids.len(), action, &err).await;
            }
        }
        Some(Popup::ProjectPicker) => {
            let ids = get_target_ids(app);
            if !ids.is_empty() {
                if app.picker_index == 0 {
                    // "None" — remove project
                    app.popup = None;
                    let action = "Project removed";
                    let nid = app.notify(NotificationKind::Loading, "Removing project...".into());
                    let mut ok = 0;
                    let mut err = String::new();
                    for id in &ids {
                        match app.client.update_issue_with_project(id, None, None, None, None, None, None, Some(None)).await {
                            Ok(_) => ok += 1,
                            Err(e) => err = e.to_string(),
                        }
                    }
                    finish_update(app, nid, ok, ids.len(), action, &err).await;
                } else if let Some(project) = app.available_projects.get(app.picker_index - 1) {
                    let project_id = project.id.clone();
                    let project_name = project.name.clone();
                    app.popup = None;
                    let action = format!("Project -> {}", project_name);
                    let nid = app.notify(NotificationKind::Loading, format!("{}...", action));
                    let mut ok = 0;
                    let mut err = String::new();
                    for id in &ids {
                        match app.client.update_issue_with_project(id, None, None, None, None, None, None, Some(Some(&project_id))).await {
                            Ok(_) => ok += 1,
                            Err(e) => err = e.to_string(),
                        }
                    }
                    finish_update(app, nid, ok, ids.len(), &action, &err).await;
                }
            }
        }
        Some(Popup::AssigneePicker) => {
            let ids = get_target_ids(app);
            if !ids.is_empty() {
                if app.picker_index == 0 {
                    // "Unassign" — need to send null assigneeId
                    app.popup = None;
                    let action = "Assignee removed";
                    let nid = app.notify(NotificationKind::Loading, "Removing assignee...".into());
                    let mut ok = 0;
                    let mut err = String::new();
                    for id in &ids {
                        match app.client.update_issue(id, None, None, None, None, Some(""), None).await {
                            Ok(_) => ok += 1,
                            Err(e) => err = e.to_string(),
                        }
                    }
                    finish_update(app, nid, ok, ids.len(), action, &err).await;
                } else if let Some(member) = app.team_members.get(app.picker_index - 1) {
                    let member_id = member.id.clone();
                    let member_name = member.name.clone();
                    app.popup = None;
                    let action = format!("Assignee -> {}", member_name);
                    let nid = app.notify(NotificationKind::Loading, format!("{}...", action));
                    let mut ok = 0;
                    let mut err = String::new();
                    for id in &ids {
                        match app.client.update_issue(id, None, None, None, None, Some(&member_id), None).await {
                            Ok(_) => ok += 1,
                            Err(e) => err = e.to_string(),
                        }
                    }
                    finish_update(app, nid, ok, ids.len(), &action, &err).await;
                }
            }
        }
        Some(Popup::BulkActions) => {
            // Bulk action selection - open the appropriate picker in bulk mode
            app.bulk_mode = true;
            match app.picker_index {
                0 => {
                    app.picker_index = 0;
                    app.popup = Some(Popup::StatusPicker);
                }
                1 => {
                    app.picker_index = 0;
                    app.popup = Some(Popup::PriorityPicker);
                }
                2 => {
                    app.picker_index = 0;
                    app.popup = Some(Popup::ProjectPicker);
                }
                3 => {
                    app.picker_index = 0;
                    app.popup = Some(Popup::LabelPicker);
                }
                4 => {
                    app.picker_index = 0;
                    app.popup = Some(Popup::AssigneePicker);
                }
                5 => {
                    // Archive selected issues
                    let issue_ids = app.get_multi_selected_issue_ids();
                    let count = issue_ids.len();
                    app.popup = None;
                    let nid = app.notify(
                        NotificationKind::Loading,
                        format!("Archiving {} issues...", count),
                    );
                    let mut success_count = 0;
                    for id in &issue_ids {
                        if app.client.archive_issue(id).await.is_ok() {
                            success_count += 1;
                        }
                    }
                    if success_count == count {
                        app.replace_notification(
                            nid,
                            NotificationKind::Success,
                            format!("Archived {} issues", count),
                        );
                    } else {
                        app.replace_notification(
                            nid,
                            NotificationKind::Error,
                            format!("Archived {}/{} issues", success_count, count),
                        );
                    }
                    app.multi_selected.clear();
                    app.bulk_mode = false;
                    let _ = app.refresh_issues().await;
                }
                _ => {
                    app.popup = None;
                    app.bulk_mode = false;
                }
            }
        }
        _ => {
            app.popup = None;
        }
    }
}

// ---------------------------------------------------------------------------
// Picker toggle — for label multi-select
// ---------------------------------------------------------------------------

fn handle_picker_toggle(app: &mut InteractiveApp) {
    if matches!(app.popup, Some(Popup::LabelPicker)) {
        if let Some(label) = app.available_labels.get(app.picker_index) {
            let label_id = label.id.clone();
            if app.selected_labels.contains(&label_id) {
                app.selected_labels.remove(&label_id);
            } else {
                app.selected_labels.insert(label_id);
            }
        }
    }
}
