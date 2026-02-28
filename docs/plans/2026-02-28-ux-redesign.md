# UX Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rewrite the interactive mode as a lazygit-style two-panel TUI with persistent detail view, toast notifications, and new features (issue creation, comments, multi-select, bulk ops, filter, assignee editing).

**Architecture:** Replace the 10-mode modal system with a 2-panel focus model (IssueList | DetailPanel) + optional popup overlay. Split monolithic ui.rs (1,567 lines) into panel/popup modules. Add a notification system for all action feedback.

**Tech Stack:** Rust, ratatui 0.26, crossterm 0.27, tokio, reqwest

---

## File Structure (Target)

```
src/interactive/
├── mod.rs              — module declarations (revised)
├── app.rs              — InteractiveApp (revised state model)
├── handlers.rs         — event loop + key dispatch (revised)
├── event.rs            — keep existing EventHandler
├── layout.rs           — panel sizing, responsive breakpoints
├── notifications.rs    — toast notification system
├── panels/
│   ├── mod.rs
│   ├── header.rs       — header bar rendering
│   ├── list.rs         — left panel (issue list)
│   └── detail.rs       — right panel (detail + comments)
├── popups/
│   ├── mod.rs
│   ├── picker.rs       — option picker (status/priority/labels/project/assignee)
│   ├── text_input.rs   — text input (comment, search, title, filter)
│   ├── confirm.rs      — confirmation dialog
│   ├── create.rs       — issue creation form
│   ├── bulk.rs         — bulk action menu
│   └── help.rs         — help overlay
└── keys.rs             — centralized keymap
```

---

## Phase 1: Cleanup & New State Model

### Task 1: Delete Dead Code

**Files:**
- Delete: `src/interactive/state.rs` (831 lines)
- Delete: `src/interactive/state_adapter.rs` (326 lines)
- Delete: `src/interactive/state_example.rs` (275 lines)
- Modify: `src/interactive/mod.rs`

**Step 1: Remove module declarations from mod.rs**

In `src/interactive/mod.rs`, remove these lines:
```rust
pub mod state;
pub mod state_adapter;

#[cfg(feature = "examples")]
pub mod state_example;
```

**Step 2: Delete the files**

```bash
rm src/interactive/state.rs src/interactive/state_adapter.rs src/interactive/state_example.rs
```

**Step 3: Build to verify nothing breaks**

```bash
cargo build 2>&1 | grep -E "^error"
```

Expected: No errors (these files were unused).

**Step 4: Commit**

```bash
git add -A && git commit -m "chore: remove 1,432 lines of dead state management code"
```

---

### Task 2: Define New State Model

**Files:**
- Modify: `src/interactive/app.rs`

Replace the current 10-variant `AppMode` enum and flat struct with a cleaner model.

**Step 1: Define new enums**

Replace the `AppMode`, `EditField`, and `GroupBy` enums at the top of `app.rs` with:

```rust
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
    ArchiveIssue(String), // issue_id
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
```

**Step 2: Update InteractiveApp struct**

Replace the struct definition with:

```rust
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
    pub multi_selected: HashSet<usize>, // indices of multi-selected issues

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

    // Create issue form
    pub create_form: CreateIssueForm,

    // Notifications
    pub notifications: Vec<Notification>,
    pub next_notification_id: u64,

    // Data
    pub client: LinearClient,
    pub workflow_states: Vec<WorkflowState>,
    pub available_labels: Vec<Label>,
    pub available_projects: Vec<Project>,
    pub team_members: Vec<User>,

    // App state
    pub should_quit: bool,
    pub loading: bool,
    pub error_message: Option<String>,

    // External editor
    pub external_editor_pending: bool,
}

#[derive(Debug, Clone, Default)]
pub struct CreateIssueForm {
    pub title: String,
    pub team_id: Option<String>,
    pub status_id: Option<String>,
    pub priority: Option<u8>,
    pub project_id: Option<String>,
    pub label_ids: Vec<String>,
    pub assignee_id: Option<String>,
    pub active_field: usize, // which field is focused
}
```

Add the import at the top of `app.rs`:

```rust
use std::collections::HashSet;
use std::time::Instant;
```

**Step 3: Stub out the Notification struct**

Add at the bottom of `app.rs` (will be moved to its own file later):

```rust
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
```

**Step 4: Update InteractiveApp::new()**

Update the constructor to initialize the new fields. Keep the existing parallel API calls but adapt to the new struct:

```rust
impl InteractiveApp {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = LinearClient::new()?;

        let (issues_result, states_result, labels_result, projects_result) = tokio::join!(
            client.get_issues(None, Some(100)),
            client.get_workflow_states(),
            client.get_labels(),
            client.get_projects(),
        );

        let issues = issues_result?;
        let workflow_states = states_result.unwrap_or_default();
        let available_labels = labels_result.unwrap_or_default();
        let available_projects = projects_result.unwrap_or_default();

        let filtered_issues = issues.clone();

        Ok(Self {
            focus: Focus::IssueList,
            popup: None,
            issues,
            filtered_issues,
            selected_index: 0,
            group_by: GroupBy::Status,
            hide_done_issues: false,
            multi_selected: HashSet::new(),
            detail_section: DetailSection::Info,
            detail_scroll: 0,
            comments: Vec::new(),
            comments_loading: false,
            last_comment_issue_id: None,
            search_query: String::new(),
            filter_query: String::new(),
            text_input: String::new(),
            text_cursor: 0,
            picker_index: 0,
            picker_search: String::new(),
            create_form: CreateIssueForm::default(),
            notifications: Vec::new(),
            next_notification_id: 0,
            client,
            workflow_states,
            available_labels,
            available_projects,
            team_members: Vec::new(),
            should_quit: false,
            loading: false,
            error_message: None,
            external_editor_pending: false,
        })
    }
}
```

**Step 5: Keep existing helper methods but adapt signatures**

Preserve these methods with same logic, adapted to new field names:
- `apply_filters()` — same logic, uses `search_query` and `hide_done_issues`
- `refresh_issues()` — same logic
- `get_selected_issue()` — returns `self.filtered_issues.get(self.selected_index)`
- `submit_comment()` — adapted to use `text_input` instead of `comment_input`
- `submit_edit()` — adapted to new popup-based flow

Remove all the old mode-specific key handler methods (they'll be rebuilt in Task 5).

**Step 6: Add notification helper methods**

```rust
impl InteractiveApp {
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
            if n.dismissed { return false; }
            match n.kind {
                NotificationKind::Success | NotificationKind::Info => {
                    now.duration_since(n.created_at).as_secs() < 5
                }
                NotificationKind::Error | NotificationKind::Loading => true,
            }
        });
    }
}
```

**Step 7: Build to check for compile errors**

```bash
cargo build 2>&1 | grep -E "^error"
```

This WILL have errors because ui.rs and handlers.rs still reference old types. That's expected — we'll fix those in the next tasks.

**Step 8: Commit**

```bash
git add src/interactive/app.rs && git commit -m "feat: new state model with Focus/Popup/Notification system"
```

---

## Phase 2: Layout & Panel Rendering

### Task 3: Create Layout Module

**Files:**
- Create: `src/interactive/layout.rs`

**Step 1: Write the layout module**

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Top-level layout regions
pub struct AppLayout {
    pub header: Rect,
    pub main: Rect,         // contains left + right panels
    pub notifications: Rect,
    pub footer: Rect,
}

/// Panel split within the main area
pub struct PanelLayout {
    pub left: Rect,  // issue list
    pub right: Rect, // detail panel
}

/// Calculate the top-level layout
pub fn app_layout(area: Rect, notification_count: usize) -> AppLayout {
    let notif_height = if notification_count > 0 {
        (notification_count as u16).min(3) + 2 // +2 for borders
    } else {
        0
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),              // header
            Constraint::Min(10),               // main panels
            Constraint::Length(notif_height),   // notifications
            Constraint::Length(3),              // footer
        ])
        .split(area);

    AppLayout {
        header: chunks[0],
        main: chunks[1],
        notifications: chunks[2],
        footer: chunks[3],
    }
}

/// Split main area into left (issue list) and right (detail) panels.
/// On narrow terminals (<100 cols), returns full width for left, zero for right.
pub fn panel_layout(area: Rect) -> PanelLayout {
    if area.width < 100 {
        // Single panel mode
        PanelLayout {
            left: area,
            right: Rect::default(),
        }
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(area);

        PanelLayout {
            left: chunks[0],
            right: chunks[1],
        }
    }
}

/// Whether we're in single-panel mode (narrow terminal)
pub fn is_single_panel(area: Rect) -> bool {
    area.width < 100
}

/// Center a popup of given width/height in the area
pub fn centered_popup(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
```

**Step 2: Register in mod.rs**

Add `pub mod layout;` to `src/interactive/mod.rs`.

**Step 3: Commit**

```bash
git add src/interactive/layout.rs src/interactive/mod.rs && git commit -m "feat: add layout module for two-panel system"
```

---

### Task 4: Create Panel Modules (Header, List, Detail)

**Files:**
- Create: `src/interactive/panels/mod.rs`
- Create: `src/interactive/panels/header.rs`
- Create: `src/interactive/panels/list.rs`
- Create: `src/interactive/panels/detail.rs`

**Step 1: Create panels/mod.rs**

```rust
pub mod header;
pub mod list;
pub mod detail;
```

**Step 2: Create panels/header.rs**

Port the header rendering from current `ui.rs` lines 230-281. The header shows the app title and issue count/filter info.

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::interactive::app::InteractiveApp;

pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(area);

    // Left: selected issue title or app name
    let title = if let Some(issue) = app.get_selected_issue() {
        format!(" {} - {} ", issue.identifier, truncate(&issue.title, (header_chunks[0].width as usize).saturating_sub(issue.identifier.len() + 6)))
    } else {
        " Linear CLI ".to_string()
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(header, header_chunks[0]);

    // Right: issue count + filter info
    let done_text = if app.hide_done_issues { " | Done: Hidden" } else { "" };
    let filter_text = if !app.filter_query.is_empty() {
        format!(" | Filter: {}", &app.filter_query)
    } else {
        String::new()
    };
    let info = format!(" Issues: {}{}{} ",
        app.filtered_issues.len(),
        done_text,
        filter_text,
    );
    let info_widget = Paragraph::new(info)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(info_widget, header_chunks[1]);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
```

**Step 3: Create panels/list.rs**

Port issue list rendering from current `ui.rs` lines 284-504. This is the left panel. Adapt to show focus border and multi-select checkmarks.

Key changes from current:
- Add focus-aware border color (Cyan when focused, DarkGray when not)
- Add `✓` prefix for multi-selected rows
- Use `►` marker for current row instead of background highlight
- Keep the existing responsive column width system
- Selected row still gets `bg(Color::Rgb(30, 35, 50))`

The file should contain:
- `pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp)` — main render function
- `struct ColumnWidths` — moved from ui.rs
- `fn calculate_column_widths(width: u16) -> ColumnWidths` — moved from ui.rs
- Helper functions: `truncate()`, `truncate_id()`, `format_age()`, `parse_assignee_name()`

Port ALL the column rendering logic from current `ui.rs` lines 32-504 into this file. The rendering should be identical to current except:
- Border is `Color::Cyan` + `BOLD` when `app.focus == Focus::IssueList`, else `Color::DarkGray`
- Multi-selected rows show `✓ ` prefix before the ID
- Block title: `" Issues "` when not focused, `" Issues (active) "` when focused

**Step 4: Create panels/detail.rs**

This is the NEW right panel that shows issue detail + comments. This replaces the old full-screen detail mode from `ui.rs` lines 535-710.

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::interactive::app::{DetailSection, Focus, InteractiveApp};

pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let focused = app.focus == Focus::DetailPanel;
    let border_style = if focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detail ")
        .border_style(border_style);

    let Some(issue) = app.get_selected_issue() else {
        let empty = Paragraph::new("No issue selected")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        frame.render_widget(empty, area);
        return;
    };

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split detail area into: info, description, comments
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),   // info fields
            Constraint::Min(5),      // description
            Constraint::Length(8),   // comments
        ])
        .split(inner);

    draw_info_section(frame, chunks[0], issue, app);
    draw_description_section(frame, chunks[1], issue, app);
    draw_comments_section(frame, chunks[2], app);
}
```

The info section shows: Title (bold cyan), Status + Priority on one line, Assignee + Project on next line, Labels below. Use the same color scheme as current detail view.

The description section renders the issue description as wrapped text. Port the existing markdown rendering from `ui.rs` lines 1320-1575 (the `render_markdown()` logic) but simplify — keep code block, header, bold, italic, link, and list rendering.

The comments section shows a scrollable list of comments with author, relative time, and body. If `app.comments_loading` is true, show a loading indicator. If no comments, show "No comments".

**Step 5: Register in mod.rs**

Add `pub mod panels;` to `src/interactive/mod.rs`.

**Step 6: Commit**

```bash
git add src/interactive/panels/ src/interactive/mod.rs && git commit -m "feat: add panel modules for header, list, and detail"
```

---

### Task 5: Create Popup Modules

**Files:**
- Create: `src/interactive/popups/mod.rs`
- Create: `src/interactive/popups/picker.rs`
- Create: `src/interactive/popups/text_input.rs`
- Create: `src/interactive/popups/confirm.rs`
- Create: `src/interactive/popups/create.rs`
- Create: `src/interactive/popups/bulk.rs`
- Create: `src/interactive/popups/help.rs`

**Step 1: Create popups/mod.rs**

```rust
pub mod picker;
pub mod text_input;
pub mod confirm;
pub mod create;
pub mod bulk;
pub mod help;

use ratatui::{Frame, layout::Rect};
use crate::interactive::app::{InteractiveApp, Popup};

/// Draw the active popup, if any.
pub fn draw_popup(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let Some(popup) = &app.popup else { return };

    match popup {
        Popup::StatusPicker | Popup::PriorityPicker |
        Popup::LabelPicker | Popup::ProjectPicker |
        Popup::AssigneePicker => picker::draw(frame, area, app),
        Popup::TextInput(_) => text_input::draw(frame, area, app),
        Popup::Confirmation(_) => confirm::draw(frame, area, app),
        Popup::CreateIssue => create::draw(frame, area, app),
        Popup::BulkActions => bulk::draw(frame, area, app),
        Popup::Help => help::draw(frame, area, app),
    }
}
```

**Step 2: Create popups/picker.rs**

Port option selection logic from current `ui.rs` lines 1060-1200. The picker shows a bordered popup with:
- Title based on picker type (e.g., "Select Status")
- Searchable list of options
- Selected option highlighted with inverted colors
- Current selection marked with `►`

Key: use `centered_popup()` from layout module for positioning.

**Step 3: Create popups/text_input.rs**

Port text input from current `ui.rs` search/comment overlays. Single-line text input with:
- Title based on context ("Add Comment", "Search", "Edit Title", "Filter")
- Text with cursor
- Submit (Enter) / Cancel (Esc) hints
- For Filter context: show autocomplete suggestions below

**Step 4: Create popups/confirm.rs**

Simple centered dialog:
```
┌─ Confirm ─────────────────┐
│ Archive issue INF-301?    │
│                           │
│    [Y]es      [N]o        │
└───────────────────────────┘
```

**Step 5: Create popups/create.rs**

Issue creation form (see design doc for layout). Tab cycles fields, Enter on dropdown opens a nested picker. Title is a text input, other fields are dropdown selectors.

**Step 6: Create popups/bulk.rs**

Simple action list shown when items are multi-selected:
- Change status / priority / project / labels / assignee / Archive
- j/k to navigate, Enter to select, Esc to cancel
- Selecting an action opens the appropriate picker for all selected issues

**Step 7: Create popups/help.rs**

Full keybinding reference overlay. Render the keybindings organized in 3 columns: Navigation, Actions, Panels. Press `?` or `Esc` to close.

**Step 8: Register in mod.rs and commit**

Add `pub mod popups;` to `src/interactive/mod.rs`.

```bash
git add src/interactive/popups/ src/interactive/mod.rs && git commit -m "feat: add popup modules for pickers, text input, create, bulk, help"
```

---

### Task 6: Create Notification Module

**Files:**
- Create: `src/interactive/notifications.rs`

**Step 1: Write notification rendering**

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::interactive::app::{InteractiveApp, NotificationKind};

pub fn draw(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    if app.notifications.is_empty() || area.height == 0 {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = app.notifications.iter()
        .filter(|n| !n.dismissed)
        .take(3)
        .map(|n| {
            let (icon, color) = match n.kind {
                NotificationKind::Success => ("✓", Color::Green),
                NotificationKind::Error => ("✗", Color::Red),
                NotificationKind::Loading => ("⟳", Color::Yellow),
                NotificationKind::Info => ("ⓘ", Color::Blue),
            };
            let elapsed = n.created_at.elapsed().as_secs();
            let timer = match n.kind {
                NotificationKind::Success | NotificationKind::Info => {
                    let remaining = 5u64.saturating_sub(elapsed);
                    format!("[{}s]", remaining)
                }
                _ => String::new(),
            };
            Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                Span::styled(&n.message, Style::default().fg(color)),
                Span::styled(format!("  {}", timer), Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
```

**Step 2: Register in mod.rs and commit**

```bash
git add src/interactive/notifications.rs src/interactive/mod.rs && git commit -m "feat: add notification rendering module"
```

---

## Phase 3: Key Handling & Main Loop

### Task 7: Create Centralized Keymap

**Files:**
- Create: `src/interactive/keys.rs`

Define all key actions as an enum and map keys to actions based on focus + popup state.

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::interactive::app::{Focus, Popup, TextInputContext};

#[derive(Debug)]
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    ScrollUp,
    ScrollDown,

    // Focus
    SwitchPanel,
    FocusDetail,
    FocusList,

    // Issue actions
    ChangeStatus,
    AddComment,
    ChangeLabels,
    ChangeProject,
    ChangeAssignee,
    EditFull,
    OpenInBrowser,
    NewIssue,
    ToggleDone,
    Refresh,
    GroupBy,

    // Multi-select
    ToggleSelect,
    ClearSelection,
    BulkAction,

    // Search / Filter
    Search,
    Filter,

    // Popup actions
    Confirm,
    Cancel,
    TypeChar(char),
    Backspace,
    MoveCursorLeft,
    MoveCursorRight,

    // Picker
    PickerUp,
    PickerDown,
    PickerConfirm,
    PickerCancel,
    PickerToggle, // for labels multi-select

    // General
    Help,
    Quit,
    DismissNotification,

    // No-op
    None,
}

pub fn map_key(key: KeyEvent, focus: &Focus, popup: &Option<Popup>) -> Action {
    // If a popup is active, route to popup-specific mappings
    if let Some(popup) = popup {
        return map_popup_key(key, popup);
    }

    // Panel-specific mappings
    match focus {
        Focus::IssueList => map_list_key(key),
        Focus::DetailPanel => map_detail_key(key),
    }
}

fn map_list_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
        KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
        KeyCode::Tab | KeyCode::Enter => Action::SwitchPanel,
        KeyCode::Char('s') => Action::ChangeStatus,
        KeyCode::Char('c') => Action::AddComment,
        KeyCode::Char('l') => Action::ChangeLabels,
        KeyCode::Char('p') => Action::ChangeProject,
        KeyCode::Char('a') => Action::ChangeAssignee,
        KeyCode::Char('e') => Action::EditFull,
        KeyCode::Char('o') => Action::OpenInBrowser,
        KeyCode::Char('n') => Action::NewIssue,
        KeyCode::Char('d') => Action::ToggleDone,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('g') => Action::GroupBy,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('f') => Action::Filter,
        KeyCode::Char('x') => Action::ToggleSelect,
        KeyCode::Char('X') => Action::ClearSelection,
        KeyCode::Char(' ') => Action::BulkAction,
        KeyCode::Char('?') => Action::Help,
        _ => Action::None,
    }
}

fn map_detail_key(key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::FocusList,
        KeyCode::Char('j') | KeyCode::Down => Action::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => Action::ScrollUp,
        KeyCode::Tab => Action::SwitchPanel,
        KeyCode::Char('s') => Action::ChangeStatus,
        KeyCode::Char('c') => Action::AddComment,
        KeyCode::Char('l') => Action::ChangeLabels,
        KeyCode::Char('p') => Action::ChangeProject,
        KeyCode::Char('a') => Action::ChangeAssignee,
        KeyCode::Char('e') => Action::EditFull,
        KeyCode::Char('o') => Action::OpenInBrowser,
        KeyCode::Char('?') => Action::Help,
        _ => Action::None,
    }
}

fn map_popup_key(key: KeyEvent, popup: &Popup) -> Action {
    match popup {
        Popup::TextInput(_) => match key.code {
            KeyCode::Enter => Action::Confirm,
            KeyCode::Esc => Action::Cancel,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Left => Action::MoveCursorLeft,
            KeyCode::Right => Action::MoveCursorRight,
            KeyCode::Char(c) => Action::TypeChar(c),
            _ => Action::None,
        },
        Popup::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => Action::Cancel,
            _ => Action::None,
        },
        Popup::Confirmation(_) => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => Action::Confirm,
            KeyCode::Char('n') | KeyCode::Esc => Action::Cancel,
            _ => Action::None,
        },
        Popup::BulkActions | Popup::CreateIssue |
        Popup::StatusPicker | Popup::PriorityPicker |
        Popup::LabelPicker | Popup::ProjectPicker |
        Popup::AssigneePicker => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::PickerDown,
            KeyCode::Char('k') | KeyCode::Up => Action::PickerUp,
            KeyCode::Enter => Action::PickerConfirm,
            KeyCode::Esc | KeyCode::Char('q') => Action::PickerCancel,
            KeyCode::Char(' ') => Action::PickerToggle,
            KeyCode::Char(c) => Action::TypeChar(c),
            KeyCode::Backspace => Action::Backspace,
            _ => Action::None,
        },
    }
}
```

**Step 2: Register and commit**

```bash
git add src/interactive/keys.rs src/interactive/mod.rs && git commit -m "feat: add centralized keymap module"
```

---

### Task 8: Rewrite handlers.rs (Main Event Loop)

**Files:**
- Modify: `src/interactive/handlers.rs`

**Step 1: Rewrite the event loop**

The main loop should:
1. Draw UI using the new panel/popup system
2. Receive key events
3. Map to actions via `keys::map_key()`
4. Dispatch actions to handler functions
5. Tick notifications on every frame
6. Fetch comments asynchronously when selected issue changes

```rust
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::config::get_api_key;
use crate::interactive::app::{
    Focus, InteractiveApp, NotificationKind, Popup, TextInputContext,
};
use crate::interactive::keys::{self, Action};
use super::event::EventHandler;

pub async fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    if api_key.is_empty() {
        eprintln!("No API key found. Run: linear auth <your-api-key>");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = InteractiveApp::new().await?;
    let events = EventHandler::new(100);

    // Track which issue we last fetched comments for
    let mut last_detail_issue_id: Option<String> = None;

    loop {
        // Tick notifications (auto-dismiss expired)
        app.tick_notifications();

        // Draw
        terminal.draw(|f| super::ui::draw(f, &app))?;

        // Fetch comments if selected issue changed
        if let Some(issue) = app.get_selected_issue() {
            let issue_id = issue.id.clone();
            if last_detail_issue_id.as_ref() != Some(&issue_id) {
                last_detail_issue_id = Some(issue_id.clone());
                app.comments_loading = true;
                app.comments.clear();
                // Spawn async comment fetch
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
        let event = events.recv()?;
        match event {
            Event::Key(key) => {
                let action = keys::map_key(key, &app.focus, &app.popup);
                handle_action(&mut app, action).await;
            }
            Event::Tick => {} // handled by notification tick above
            _ => {}
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_action(app: &mut InteractiveApp, action: Action) {
    match action {
        // Navigation
        Action::MoveUp => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
                app.detail_scroll = 0;
            }
        }
        Action::MoveDown => {
            if app.selected_index < app.filtered_issues.len().saturating_sub(1) {
                app.selected_index += 1;
                app.detail_scroll = 0;
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
                Focus::IssueList => Focus::DetailPanel,
                Focus::DetailPanel => Focus::IssueList,
            };
        }
        Action::FocusList => { app.focus = Focus::IssueList; }
        Action::FocusDetail => { app.focus = Focus::DetailPanel; }

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
            app.create_form = Default::default();
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

        // Popup actions: Confirm/Cancel
        Action::Confirm => handle_confirm(app).await,
        Action::Cancel => { app.popup = None; }

        // Text input
        Action::TypeChar(c) => {
            app.text_input.insert(app.text_cursor, c);
            app.text_cursor += 1;
        }
        Action::Backspace => {
            if app.text_cursor > 0 {
                app.text_cursor -= 1;
                app.text_input.remove(app.text_cursor);
            }
        }
        Action::MoveCursorLeft => {
            app.text_cursor = app.text_cursor.saturating_sub(1);
        }
        Action::MoveCursorRight => {
            if app.text_cursor < app.text_input.len() {
                app.text_cursor += 1;
            }
        }

        // Picker navigation
        Action::PickerUp => {
            app.picker_index = app.picker_index.saturating_sub(1);
        }
        Action::PickerDown => {
            app.picker_index += 1; // clamped during render
        }
        Action::PickerConfirm => handle_picker_confirm(app).await,
        Action::PickerCancel => { app.popup = None; }
        Action::PickerToggle => handle_picker_toggle(app),

        // Other
        Action::OpenInBrowser => {
            if let Some(issue) = app.get_selected_issue() {
                let _ = open::that(&issue.url);
            }
        }
        Action::ToggleDone => {
            app.hide_done_issues = !app.hide_done_issues;
            app.apply_filters();
        }
        Action::GroupBy => {
            app.group_by = match app.group_by {
                super::app::GroupBy::Status => super::app::GroupBy::Project,
                super::app::GroupBy::Project => super::app::GroupBy::Status,
            };
            app.apply_filters();
        }
        Action::Refresh => {
            let nid = app.notify(NotificationKind::Loading, "Refreshing issues...".into());
            match app.refresh_issues().await {
                Ok(_) => app.replace_notification(nid, NotificationKind::Success, "Issues refreshed".into()),
                Err(e) => app.replace_notification(nid, NotificationKind::Error, format!("Refresh failed: {}", e)),
            }
        }
        Action::Quit => { app.should_quit = true; }
        Action::DismissNotification => {
            // Dismiss the oldest error notification
            if let Some(n) = app.notifications.iter_mut().find(|n| n.kind == NotificationKind::Error && !n.dismissed) {
                n.dismissed = true;
            }
        }
        Action::EditFull => {
            // TODO: implement full edit menu as a popup
        }
        Action::None => {}
    }
}
```

Add `handle_confirm()` and `handle_picker_confirm()` as separate async functions that:
- Read the popup type from `app.popup`
- Execute the appropriate API call with loading/success/error notifications
- Close the popup on completion
- Refresh the issue list

**Step 2: Build and fix compile errors**

This will require iterating on imports, method signatures, etc. The goal is a compiling handler module.

**Step 3: Commit**

```bash
git add src/interactive/handlers.rs && git commit -m "feat: rewrite event loop with action-based dispatch"
```

---

### Task 9: Rewrite ui.rs as Compositor

**Files:**
- Modify: `src/interactive/ui.rs`

Replace the 1,567-line monolith with a thin compositor that calls the panel/popup modules.

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::interactive::app::InteractiveApp;
use crate::interactive::layout;

pub fn draw(frame: &mut Frame, app: &InteractiveApp) {
    let area = frame.area();

    // Calculate layout
    let app_layout = layout::app_layout(area, app.notifications.len());
    let panels = layout::panel_layout(app_layout.main);

    // Header
    super::panels::header::draw(frame, app_layout.header, app);

    // Left panel: issue list
    super::panels::list::draw(frame, panels.left, app);

    // Right panel: detail (only in two-panel mode)
    if panels.right.width > 0 {
        super::panels::detail::draw(frame, panels.right, app);
    }

    // Notifications
    if app_layout.notifications.height > 0 {
        super::notifications::draw(frame, app_layout.notifications, app);
    }

    // Footer (action hints)
    draw_footer(frame, app_layout.footer, app);

    // Popup overlay (drawn last, on top of everything)
    super::popups::draw_popup(frame, area, app);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &InteractiveApp) {
    let help_text = if app.popup.is_some() {
        "" // Popup has its own hints
    } else if !app.multi_selected.is_empty() {
        "[Space] Bulk action  [x] Toggle select  [X] Clear  [Esc] Cancel"
    } else {
        "[s]tatus [c]omment [l]abels [p]roject [a]ssign [e]dit [n]ew [/]search [f]ilter [?]help"
    };

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::LightGreen))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, area);
}
```

**Step 1: Replace the entire contents of ui.rs with the compositor above**

**Step 2: Build, fix any remaining import issues**

**Step 3: Commit**

```bash
git add src/interactive/ui.rs && git commit -m "feat: replace monolithic ui.rs with thin compositor"
```

---

## Phase 4: Integration & Polish

### Task 10: Wire Up Comment Fetching

**Files:**
- Modify: `src/interactive/handlers.rs`
- Modify: `src/interactive/panels/detail.rs`

The comment fetching in the event loop needs to be non-blocking. Use `tokio::spawn` to fetch comments in the background and update the app state when done. For now, the simpler approach (await in the loop) works since the API is fast, but add a loading indicator.

**Step 1: Add comment cache logic**

In `handlers.rs`, the comment fetch should only trigger when the selected issue changes:
- Track `last_detail_issue_id`
- When it changes, set `comments_loading = true`, clear `comments`
- Fetch and update

**Step 2: Render comments in detail panel**

Ensure `panels/detail.rs` shows:
- "Loading comments..." (yellow) when `comments_loading`
- "No comments" (gray) when empty
- Formatted comment list otherwise

**Step 3: Commit**

```bash
git commit -am "feat: wire up live comment fetching in detail panel"
```

---

### Task 11: Wire Up All Picker Actions (Status, Priority, Labels, Project, Assignee)

**Files:**
- Modify: `src/interactive/handlers.rs` (handle_picker_confirm, handle_picker_toggle)
- Modify: `src/interactive/popups/picker.rs`

**Step 1: Implement `handle_picker_confirm()`**

For each picker type, execute the appropriate API call:
- `StatusPicker` → `client.update_issue(id, ..., state_id, ...)`
- `PriorityPicker` → `client.update_issue(id, ..., priority, ...)`
- `LabelPicker` → `client.update_issue(id, ..., label_ids, ...)`
- `ProjectPicker` → `client.update_issue_with_project(id, ..., project_id)`
- `AssigneePicker` → `client.update_issue(id, ..., assignee_id, ...)`

Each follows the pattern:
1. Show loading notification
2. Make API call
3. Replace notification with success/error
4. Close popup
5. Refresh issues

**Step 2: Implement `handle_picker_toggle()`**

Only used for LabelPicker (multi-select labels with Space).

**Step 3: Implement `handle_confirm()` for TextInput and Confirmation popups**

- `TextInput(Comment)` → `client.create_comment(issue_id, body)`
- `TextInput(Search)` → set `app.search_query`, call `apply_filters()`
- `TextInput(Filter)` → set `app.filter_query`, apply filter parser
- `TextInput(EditTitle)` → `client.update_issue(id, title, ...)`
- `Confirmation(ArchiveIssue)` → `client.archive_issue(id)`

**Step 4: Commit**

```bash
git commit -am "feat: wire up all picker and text input actions with API calls"
```

---

### Task 12: Issue Creation Flow

**Files:**
- Modify: `src/interactive/popups/create.rs`
- Modify: `src/interactive/handlers.rs`

**Step 1: Implement create form rendering**

The create popup shows fields with Tab navigation. Title is a text field, others are dropdown selectors. When a dropdown field is active and Enter is pressed, open a nested picker.

**Step 2: Implement create submission**

On final Enter (when Title is non-empty):
1. Loading notification
2. `client.create_issue(title, description, team_id, priority, assignee_id, label_ids)`
3. Success/error notification
4. Close popup, refresh issues

**Step 3: Add team member fetching**

The AssigneePicker and CreateIssue form need team members. Add a `client.get_team_members()` call. Currently this method doesn't exist in the client — it needs to be added:

In `src/client/linear_client.rs`, add:
```rust
pub async fn get_team_members(&self) -> Result<Vec<User>, Box<dyn std::error::Error>> {
    // Query: { teams { nodes { members { nodes { id name email } } } } }
    // Flatten all team members, deduplicate by id
}
```

**Step 4: Commit**

```bash
git commit -am "feat: add issue creation flow with team member fetching"
```

---

### Task 13: Bulk Operations

**Files:**
- Modify: `src/interactive/popups/bulk.rs`
- Modify: `src/interactive/handlers.rs`

**Step 1: Implement bulk action menu rendering**

Show action list: Change status, priority, project, labels, assignee, Archive.

**Step 2: Implement bulk action dispatch**

When an action is selected:
1. Close BulkActions popup
2. Open the appropriate picker
3. On picker confirm, apply to ALL multi-selected issues
4. Use `client.update_issue_bulk()` for efficiency where possible
5. Show loading → success notification with count: "✓ Updated 3 issues"
6. Clear multi-selection

**Step 3: Commit**

```bash
git commit -am "feat: add bulk operations for multi-selected issues"
```

---

### Task 14: Final Polish & Testing

**Files:**
- All interactive modules

**Step 1: Test all key flows manually**

- Normal navigation (j/k, Tab between panels)
- Status change with notification feedback
- Comment add with notification feedback
- Issue creation
- Multi-select + bulk status change
- Search and filter
- Help overlay
- Error handling (disconnect network, try an action)

**Step 2: Fix any visual glitches**

- Column alignment
- Popup positioning
- Notification overflow
- Narrow terminal fallback

**Step 3: Build release and install**

```bash
cargo build --release && cargo install --path .
```

**Step 4: Final commit**

```bash
git commit -am "feat: UX redesign complete — lazygit-style two-panel layout"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-2 | Cleanup dead code, define new state model |
| 2 | 3-6 | Layout, panels, popups, notifications |
| 3 | 7-9 | Keymap, event loop, ui compositor |
| 4 | 10-14 | Wire up features, polish, test |

Total: 14 tasks. The app will be non-functional between Tasks 2-9 (state model doesn't match rendering). Tasks 1-9 should be done as one batch before testing.
