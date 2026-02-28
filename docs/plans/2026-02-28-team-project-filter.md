# Team & Project Filter Sidebar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add lazygit-style Teams and Projects selector boxes above the issue list, with server-side re-fetching when a team or project is selected.

**Architecture:** Extend the Focus enum from 2 to 4 variants (TeamList, ProjectList, IssueList, DetailPanel). Add two new panel renderers for the selector boxes. Split the left column layout into 3 vertical zones. On team/project selection, rebuild the API filter and re-fetch issues.

**Tech Stack:** Rust, ratatui 0.26, crossterm 0.27, tokio, reqwest, serde_json

---

## Task 1: Extend State Model

**Files:**
- Modify: `src/interactive/app.rs`

**Step 1: Extend the Focus enum**

In `src/interactive/app.rs`, replace the Focus enum:

```rust
/// Which panel has keyboard focus
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    TeamList,
    ProjectList,
    IssueList,
    DetailPanel,
}
```

**Step 2: Add new state fields to InteractiveApp**

Add these fields to the `InteractiveApp` struct, in a new section after `// Issue list state`:

```rust
    // Team & project selectors
    pub teams: Vec<crate::models::Team>,
    pub active_team: Option<usize>,    // index into teams (None = all teams)
    pub active_project: Option<usize>, // index into available_projects (0 = "All", 1+ = project)
    pub team_index: usize,             // cursor position in teams box
    pub project_index: usize,          // cursor position in projects box
```

**Step 3: Initialize new fields in `InteractiveApp::new()`**

In the constructor, add the new fields to the struct literal (default values):

```rust
            // Team & project selectors
            teams: Vec::new(),
            active_team: None,
            active_project: None,
            team_index: 0,
            project_index: 0,
```

**Step 4: Fetch teams at startup**

Teams are already fetched via `get_teams()` but the result isn't stored. The `get_teams()` call needs to be added to the `tokio::join!` block. Currently it joins 5 futures (issues, states, labels, projects, members). Add teams:

```rust
        let (issues_result, states_result, labels_result, projects_result, members_result, teams_result) = tokio::join!(
            app.client.get_issues(None, Some(100)),
            app.client.get_workflow_states(),
            app.client.get_labels(),
            app.client.get_projects(),
            app.client.get_team_members(),
            app.client.get_teams()
        );
```

Add a handler after the team members handler:

```rust
        // Handle teams result
        match teams_result {
            Ok(teams) => {
                app.teams = teams;
            }
            Err(e) => {
                log_error(&format!("Failed to fetch teams: {}", e));
                app.teams = Vec::new();
            }
        }
```

**Step 5: Add a filter-building helper**

Add this method to `impl InteractiveApp`:

```rust
    /// Build the GraphQL IssueFilter based on active team and project selections
    pub fn build_issue_filter(&self) -> Option<serde_json::Value> {
        let mut filter = serde_json::json!({});
        let mut has_filter = false;

        if let Some(team_idx) = self.active_team {
            if let Some(team) = self.teams.get(team_idx) {
                filter["team"] = serde_json::json!({"id": {"eq": team.id}});
                has_filter = true;
            }
        }

        if let Some(proj_idx) = self.active_project {
            // Index 0 = "All", so only filter for index >= 1
            if proj_idx > 0 {
                if let Some(project) = self.available_projects.get(proj_idx - 1) {
                    filter["project"] = serde_json::json!({"id": {"eq": project.id}});
                    has_filter = true;
                }
            }
        }

        if has_filter { Some(filter) } else { None }
    }
```

**Step 6: Update `refresh_issues()` to use the filter**

Replace the hardcoded `None` filter in `refresh_issues()`:

```rust
    pub async fn refresh_issues(&mut self) -> Result<(), Box<dyn Error>> {
        self.loading = true;
        self.error_message = None;

        let filter = self.build_issue_filter();
        match self.client.get_issues(filter, Some(100)).await {
```

**Step 7: Update the initial Focus default**

Change the default focus from `IssueList` to `TeamList` so the app starts with focus on the teams box:

```rust
            focus: Focus::IssueList,
```

Keep it as `IssueList` — users will land on the issue list by default, which is the most useful starting point. They can Tab/Shift-Tab to the team/project boxes when needed.

**Step 8: Commit**

```bash
git add src/interactive/app.rs
git commit -m "feat: extend state model with team/project selector fields"
```

---

## Task 2: Update Layout

**Files:**
- Modify: `src/interactive/layout.rs`

**Step 1: Add LeftColumnLayout struct**

Add a new struct for the left column's 3-zone split:

```rust
/// Left column split: teams, projects, issues
pub struct LeftColumnLayout {
    pub teams: Rect,
    pub projects: Rect,
    pub issues: Rect,
}
```

**Step 2: Add left_column_layout function**

```rust
/// Split the left column into teams box, projects box, and issue list.
/// Teams and projects get fixed height based on item count (max 5 rows + 2 for borders).
/// Issues get the remaining space.
pub fn left_column_layout(area: Rect, team_count: usize, project_count: usize) -> LeftColumnLayout {
    // Each box needs item_count rows + 2 for borders, capped at 7 (5 visible + 2 borders)
    let teams_height = ((team_count as u16) + 2).min(7).max(3);
    let projects_height = ((project_count as u16) + 2).min(7).max(3);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(teams_height),
            Constraint::Length(projects_height),
            Constraint::Min(5),
        ])
        .split(area);

    LeftColumnLayout {
        teams: chunks[0],
        projects: chunks[1],
        issues: chunks[2],
    }
}
```

**Step 3: Commit**

```bash
git add src/interactive/layout.rs
git commit -m "feat: add left column layout with team/project/issue zones"
```

---

## Task 3: Create Team & Project Panel Renderers

**Files:**
- Create: `src/interactive/panels/teams.rs`
- Create: `src/interactive/panels/projects.rs`
- Modify: `src/interactive/panels/mod.rs`

**Step 1: Create `panels/teams.rs`**

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::interactive::app::{Focus, InteractiveApp};

pub fn draw_teams(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::TeamList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(" Teams ({}) ", app.teams.len());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if app.teams.is_empty() {
        let empty = ratatui::widgets::Paragraph::new("No teams")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll_offset = if app.team_index >= inner_height {
        app.team_index - inner_height + 1
    } else {
        0
    };

    let items: Vec<ListItem> = app
        .teams
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|(i, team)| {
            let marker = if app.active_team == Some(i) { "►" } else { " " };
            let display = format!("{} {} ({})", marker, team.name, team.key);

            let style = if i == app.team_index && focused {
                Style::default()
                    .bg(Color::Rgb(30, 35, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if app.active_team == Some(i) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
```

**Step 2: Create `panels/projects.rs`**

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::interactive::app::{Focus, InteractiveApp};

pub fn draw_projects(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::ProjectList;
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // +1 for the "All" option
    let count = app.available_projects.len() + 1;
    let title = format!(" Projects ({}) ", count);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize;
    let scroll_offset = if app.project_index >= inner_height {
        app.project_index - inner_height + 1
    } else {
        0
    };

    // Build options: "All" at index 0, then each project
    let mut options: Vec<(usize, String)> = vec![(0, "All".to_string())];
    options.extend(
        app.available_projects
            .iter()
            .enumerate()
            .map(|(i, p)| (i + 1, p.name.clone())),
    );

    let items: Vec<ListItem> = options
        .iter()
        .skip(scroll_offset)
        .take(inner_height)
        .map(|(idx, name)| {
            let is_active = match app.active_project {
                None => *idx == 0,     // None means "All" is active
                Some(ap) => ap == *idx,
            };
            let marker = if is_active { "►" } else { " " };
            let display = format!("{} {}", marker, name);

            let style = if *idx == app.project_index && focused {
                Style::default()
                    .bg(Color::Rgb(30, 35, 50))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_active {
                Style::default().fg(Color::LightGreen)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(Line::from(Span::styled(display, style)))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
```

**Step 3: Update `panels/mod.rs`**

```rust
pub mod header;
pub mod list;
pub mod detail;
pub mod teams;
pub mod projects;
```

**Step 4: Commit**

```bash
git add src/interactive/panels/teams.rs src/interactive/panels/projects.rs src/interactive/panels/mod.rs
git commit -m "feat: add team and project panel renderers"
```

---

## Task 4: Update UI Compositor

**Files:**
- Modify: `src/interactive/ui.rs`

**Step 1: Update `draw()` to use left column layout**

Replace the current `draw()` function body in `ui.rs`. The key change is splitting the left panel into 3 zones using `left_column_layout()`:

```rust
pub fn draw(frame: &mut Frame, app: &InteractiveApp) {
    let area = frame.size();

    // Calculate layout
    let active_notifs = app.notifications.iter().filter(|n| !n.dismissed).count();
    let app_layout = layout::app_layout(area, active_notifs);
    let panels = layout::panel_layout(app_layout.main);

    // Header
    super::panels::header::draw_header(frame, app_layout.header, app);

    // Left column: teams, projects, issues
    let left_col = layout::left_column_layout(
        panels.left,
        app.teams.len(),
        app.available_projects.len() + 1, // +1 for "All"
    );
    super::panels::teams::draw_teams(frame, left_col.teams, app);
    super::panels::projects::draw_projects(frame, left_col.projects, app);
    super::panels::list::draw_list(frame, left_col.issues, app);

    // Right panel: detail (only in two-panel mode)
    if panels.right.width > 0 {
        super::panels::detail::draw_detail(frame, panels.right, app);
    }

    // Notifications
    if app_layout.notifications.height > 0 {
        super::notifications::draw(frame, app_layout.notifications, app);
    }

    // Footer
    draw_footer(frame, app_layout.footer, app);

    // Popup overlay (drawn last, on top of everything)
    super::popups::draw_popup(frame, area, app);
}
```

**Step 2: Commit**

```bash
git add src/interactive/ui.rs
git commit -m "feat: wire up team/project panels in ui compositor"
```

---

## Task 5: Update Keymap

**Files:**
- Modify: `src/interactive/keys.rs`

**Step 1: Add new Action variants**

Add to the `Action` enum:

```rust
    // Team/Project selection
    SelectTeam,
    SelectProject,
```

**Step 2: Add key handlers for TeamList and ProjectList focus**

In `map_key()`, expand the focus match to handle the new focus variants:

```rust
pub fn map_key(key: KeyEvent, focus: &Focus, popup: &Option<Popup>) -> Action {
    if let Some(popup) = popup {
        return map_popup_key(key, popup);
    }
    match focus {
        Focus::TeamList => map_team_key(key),
        Focus::ProjectList => map_project_key(key),
        Focus::IssueList => map_list_key(key),
        Focus::DetailPanel => map_detail_key(key),
    }
}
```

**Step 3: Implement map_team_key and map_project_key**

```rust
fn map_team_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::SelectTeam,
        KeyCode::Tab => Action::SwitchPanel,
        KeyCode::BackTab => Action::FocusList, // Shift-Tab wraps to detail (handled in handler)
        KeyCode::Char('?') => Action::Help,
        KeyCode::Char('r') => Action::Refresh,
        _ => Action::None,
    }
}

fn map_project_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Enter => Action::SelectProject,
        KeyCode::Tab => Action::SwitchPanel,
        KeyCode::BackTab => Action::FocusList, // Shift-Tab (handled in handler)
        KeyCode::Char('?') => Action::Help,
        KeyCode::Char('r') => Action::Refresh,
        _ => Action::None,
    }
}
```

**Step 4: Commit**

```bash
git add src/interactive/keys.rs
git commit -m "feat: add keymap for team and project focus states"
```

---

## Task 6: Update Event Handlers

**Files:**
- Modify: `src/interactive/handlers.rs`

**Step 1: Update MoveUp/MoveDown for team/project focus**

In `handle_action()`, update the `MoveUp` and `MoveDown` arms to handle the new focus states:

```rust
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
```

Remove the separate `Action::ScrollUp` / `Action::ScrollDown` arms — they're now handled inside MoveUp/MoveDown for DetailPanel focus.

**Step 2: Update SwitchPanel (Tab) to cycle through 4 zones**

```rust
        Action::SwitchPanel => {
            app.focus = match app.focus {
                Focus::TeamList => Focus::ProjectList,
                Focus::ProjectList => Focus::IssueList,
                Focus::IssueList => Focus::DetailPanel,
                Focus::DetailPanel => Focus::TeamList,
            };
        }
```

**Step 3: Update FocusList for Shift-Tab / Esc**

```rust
        Action::FocusList => {
            // Shift-Tab: go backwards in focus cycle
            app.focus = match app.focus {
                Focus::TeamList => Focus::DetailPanel,
                Focus::ProjectList => Focus::TeamList,
                Focus::IssueList => Focus::ProjectList,
                Focus::DetailPanel => Focus::IssueList,
            };
        }
```

**Step 4: Add SelectTeam handler**

```rust
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
                let team_name = app.teams.get(app.team_index)
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
```

**Step 5: Add SelectProject handler**

```rust
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
                    let name = app.available_projects.get(app.project_index - 1)
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
```

**Step 6: Commit**

```bash
git add src/interactive/handlers.rs
git commit -m "feat: add team/project selection handlers with server-side re-fetch"
```

---

## Task 7: Update Issue List Panel Focus Check

**Files:**
- Modify: `src/interactive/panels/list.rs`

**Step 1: Update the focus check**

In `draw_list()`, the border focus check currently uses `app.focus == Focus::IssueList`. This is still correct — no change needed. But verify the selected row highlight only applies when the issue list is focused. Find the line that checks focus for the selected row styling and confirm it checks `Focus::IssueList`.

**Step 2: Commit (if changes needed)**

```bash
git add src/interactive/panels/list.rs
git commit -m "fix: update list panel focus check for new focus variants"
```

---

## Task 8: Build, Test, and Polish

**Files:**
- All modified files

**Step 1: Build and verify**

```bash
cargo build 2>&1 | grep "^error"
```

Fix any compilation errors.

**Step 2: Install and test**

```bash
cargo install --path .
```

Test manually:
- App starts with issue list focused, teams and projects boxes visible above
- Tab cycles: Issues → Detail → Teams → Projects → Issues
- Shift-Tab cycles backwards
- j/k navigates within focused box
- Enter in Teams box selects team, shows loading notification, re-fetches issues
- Enter on already-selected team deselects (shows all teams)
- Enter in Projects box selects project, re-fetches with combined filter
- "All" in projects clears project filter
- Team change resets project to "All"

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: team/project filter sidebar complete"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Extend state model | app.rs |
| 2 | Update layout | layout.rs |
| 3 | Create panel renderers | panels/teams.rs, panels/projects.rs, panels/mod.rs |
| 4 | Update UI compositor | ui.rs |
| 5 | Update keymap | keys.rs |
| 6 | Update event handlers | handlers.rs |
| 7 | Update list panel focus | panels/list.rs |
| 8 | Build, test, polish | all |

Total: 8 tasks. All tasks can be done sequentially. The app will compile after Task 6 (all pieces connected).
