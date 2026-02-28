# UX Redesign: Lazygit-Style Panel Layout

## Goals

- Replace single-list-with-modals UI with a persistent two-panel layout
- Fix the entire feedback loop: loading indicators, success/error toasts, action confirmations
- Add missing features: issue creation, comment viewing, multi-select, assignee editing, filter integration
- Clean up ~1,400 lines of dead code and split the monolithic ui.rs

Inspired by lazygit. Big-bang rewrite of the interactive mode.

## Layout

```
┌─ Header ──────────────────────────────────────────────────────┐
│ Linear CLI    Issues: 47 | Filter: assignee:me priority:>1    │
├─ Issues (left, ~40%) ─────────┬─ Detail (right, ~60%) ────────┤
│ ID      P Title        Status │ INF-301: Creator Updates      │
│ INF-301 ■ Creator Up.. Backlog│ ──────────────────────────────│
│ INF-300 • Dashboard..  Backlog│ Status: Backlog  Priority: 3  │
│►INF-219   Add Posts..  Cancel │ Assignee: cole                │
│ INF-276 ■ Upgrade o..  Deploy │ Project: ABLE  Labels: Front  │
│ INF-253 ■ Fix excess   Deploy │ ──────────────────────────────│
│                               │ Description:                  │
│                               │ This task involves updating   │
│                               │ the creator dashboard with... │
│                               │ ──────────────────────────────│
│                               │ Comments (3):                 │
│                               │ cole (2d): Looking into this  │
│                               │ egor (1d): Can we prioritize? │
├─ Notifications ───────────────┴───────────────────────────────┤
│ ✓ Status updated to "In Progress"                       [3s]  │
├─ Actions ─────────────────────────────────────────────────────┤
│ [s]tatus [c]omment [l]abels [p]roject [e]dit [n]ew [/]search │
└───────────────────────────────────────────────────────────────┘
```

- Left panel: issue list, always visible, never hidden by modals
- Right panel: detail + comments for selected issue, updates live as you j/k navigate
- Notification bar: toast messages (success/error/loading). Max 3 visible.
- Action bar: context-aware keybinding hints
- Narrow terminals (<100 cols): collapse to single panel, Enter opens detail as overlay

## Navigation

Two-panel focus system. One panel active at a time, indicated by highlighted border.

### Left Panel (Issues) Focus

| Key | Action |
|-----|--------|
| j/k | Navigate issues |
| / | Search/filter |
| g | Group by |
| n | New issue |
| d | Toggle done |
| r | Refresh |
| x | Toggle multi-select |
| X | Clear multi-selection |
| Space | Bulk action menu (when items selected) |
| Tab | Focus right panel |
| Enter | Focus right panel |

### Right Panel (Detail) Focus

| Key | Action |
|-----|--------|
| j/k | Scroll description/comments |
| Tab | Cycle sections (info, description, comments) or focus left panel |
| o | Open in browser |
| 0-9 | Open numbered link |
| Esc | Focus left panel |

### Global Keys (any focus)

| Key | Action |
|-----|--------|
| s | Change status (picker popup) |
| c | Add comment (text input popup) |
| l | Change labels (picker popup) |
| p | Change project (picker popup) |
| a | Change assignee (picker popup) |
| e | Full edit menu |
| ? | Help overlay |
| q | Quit |

## Popups

Popups only for input. Three types:

1. **Text Input** (centered): comment, title edit, search query, filter query
2. **Option Picker** (anchored to active panel): status, priority, labels, project, assignee
3. **Confirmation** (small, centered): delete, discard changes

## Notification System

```rust
struct Notification {
    id: u64,
    kind: NotificationKind,
    message: String,
    created_at: Instant,
    dismissed: bool,
}

enum NotificationKind {
    Success,    // green ✓, auto-fade 5s
    Error,      // red ✗, persists until dismissed with 'x'
    Loading,    // yellow ⟳, replaced when API call completes
    Info,       // blue ⓘ, auto-fade 5s
}
```

Every action follows: trigger → loading toast → success/error toast.

Max 3 visible notifications. Oldest pushed out. Countdown timer shown on auto-dismiss toasts.

## New Features

### Issue Creation (n key)

Centered popup form:
- Title (required), Team, Status, Priority, Project, Labels, Assignee
- Tab cycles fields, dropdown fields open option picker
- On submit: closes → loading toast → success toast → issue appears in list

### Comment Viewing

Comments shown in the bottom section of the right panel:
- Loaded on demand when issue is selected
- Loading indicator while fetching
- Scrollable when right panel focused
- Shows author, relative time, and body

### Multi-Select & Bulk Operations (x / Space)

- x toggles checkmark on current row
- X clears all selections
- Info toast shows count: "2 issues selected — Space for bulk actions"
- Space opens bulk action menu: change status, priority, project, labels, assignee, archive

### Filter Integration (f key)

Wire up the existing filter parser (888 lines, already built):
- Overlay at top of issue list
- Live filtering as you type
- Autocomplete suggestions for field names and values
- Active filter shown in header bar

### Assignee Editing (a key)

Picker popup with team member list:
- Search/filter team members
- "Unassign" option at bottom

### Help Overlay (? key)

Full-screen overlay showing all keybindings organized by category:
Navigation, Actions, Panels. Press ? or Esc to close.

## Technical Changes

### Dead Code Removal

Delete:
- `src/interactive/state.rs` (830 lines)
- `src/interactive/state_adapter.rs` (325 lines)
- `src/interactive/state_example.rs` (275 lines)

### File Structure

Split `ui.rs` (1,567 lines) into:
- `panels/list.rs` — issue list rendering
- `panels/detail.rs` — detail + comments panel
- `panels/header.rs` — header bar
- `popups/mod.rs` — all popup types
- `notifications.rs` — toast system
- `layout.rs` — panel sizing, responsive breakpoints

### Color Rules

- Never use `bg(Color::Black)` — always use terminal default
- Only use `Color::Rgb()` for explicit backgrounds (selected row, popups)
- Selected row: `Color::Rgb(30, 35, 50)` (dark navy, bypasses ANSI remapping)
- Popup backgrounds: `Color::Rgb(25, 25, 30)`

### State Simplification

Replace 10 AppModes with:
- `Focus` enum: `IssueList | DetailPanel`
- `Popup` enum (optional): `None | StatusPicker | LabelPicker | ... | TextInput | Confirmation`
- Active popup overlays on top of the two-panel layout
- No more `previous_mode` tracking — closing a popup returns to the focused panel

### Other

- Centralize keybindings in a `KeyMap` struct
- Fix hardcoded 100-issue limit — add pagination or lazy loading
- Cache comments per issue to avoid re-fetching
