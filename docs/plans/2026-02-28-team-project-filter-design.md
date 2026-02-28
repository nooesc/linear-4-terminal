# Team & Project Filter Sidebar

## Goals

- Add a Teams box and Projects box above the issue list in the left column
- Selecting a team or project re-fetches issues server-side from the Linear API
- Lazygit-style stacked focus zones: Teams → Projects → Issues → Detail

## Layout

```
┌─ Header ──────────────────────────────────────────────────────┐
│ Linear CLI                    Issues: 24 | Group: Status      │
├─ Teams (2) ──────────┬─ Detail ──────────────────────────────┤
│ ► ABLE               │ INF-301: Creator Updates              │
│   Backend            │ ──────────────────────────────────────│
├─ Projects (3) ───────┤ Status: Backlog  Priority: 3          │
│   All                │ Assignee: cole                        │
│ ► Creator v2         │ ...                                   │
│   Dashboard          │                                       │
├─ Issues (24) ────────┤                                       │
│ INF-301 ■ Creator..  │                                       │
│ INF-300 • Dash..     │                                       │
│►INF-219   Add..      │                                       │
│ INF-276 ■ Upgrade..  │                                       │
├─ Notifications ──────┴───────────────────────────────────────┤
│ ✓ Switched to team ABLE                                      │
├─ Actions ────────────────────────────────────────────────────┤
│ [s]tatus [c]omment [l]abels [p]roject [a]ssign [n]ew [?]help │
└──────────────────────────────────────────────────────────────┘
```

- Teams box: Fixed height based on team count (max 5 visible rows, scrollable).
- Projects box: Fixed height based on project count (max 5 visible rows). First item is always "All" (no project filter).
- Issues box: Takes remaining vertical space.
- Active items marked with `►`. Focused box has cyan border, unfocused has dark gray.
- Narrow terminals (<100 cols): Boxes still appear above issue list; detail panel collapses.

## Navigation

Focus enum expands to 4 variants:

```
Focus::TeamList → Focus::ProjectList → Focus::IssueList → Focus::DetailPanel
     ↑                                                          │
     └──────────────────── Shift-Tab ───────────────────────────┘
```

| Key | Teams box | Projects box | Issues box | Detail panel |
|-----|-----------|-------------|------------|--------------|
| j/k | Navigate teams | Navigate projects | Navigate issues | Scroll |
| Enter | Select team (re-fetch) | Select project (re-fetch) | Focus detail | — |
| Tab | → Projects | → Issues | → Detail | → Teams |
| Shift-Tab | ← Detail | ← Teams | ← Projects | ← Issues |

- Global keys (s, c, l, p, a, n, ?, q) work from any focus.
- When team changes, project selection resets to "All".

## Data Flow

### Startup
1. Fetch teams, projects, issues (100), labels, workflow states, team members — all in parallel.
2. Default: no team filter, no project filter (show all issues).
3. Teams box shows all teams. No team pre-selected.
4. Projects box shows "All" + all loaded projects.

### Select Team
1. Set `active_team` to selected team index.
2. Show loading notification.
3. Call `get_issues(Some(filter), 100)` where filter = `{"team": {"id": {"eq": team_id}}}`.
4. Replace issue list, reset selection to 0, reset project to "All".
5. Success/error notification.

### Select Project
1. Set `active_project` to selected project index (0 = "All").
2. Show loading notification.
3. Build combined filter: team filter (if active) AND project filter (if not "All").
   - Project filter: `{"project": {"id": {"eq": project_id}}}`
4. Call `get_issues(Some(combined_filter), 100)`.
5. Replace issue list, reset selection to 0.
6. Success/error notification.

## State Changes

New fields in `InteractiveApp`:
```rust
pub teams: Vec<Team>,              // from get_teams()
pub active_team: Option<usize>,    // index into teams (None = all teams)
pub active_project: Option<usize>, // index into available_projects (None/0 = all)
pub team_index: usize,             // cursor position in teams box
pub project_index: usize,          // cursor position in projects box
```

`Focus` enum:
```rust
pub enum Focus {
    TeamList,
    ProjectList,
    IssueList,
    DetailPanel,
}
```

New `Action` variants:
```rust
Action::SelectTeam,     // Enter in TeamList focus
Action::SelectProject,  // Enter in ProjectList focus
```

## Files to Modify

- `app.rs` — New state fields, extend Focus enum, add `build_issue_filter()` helper
- `layout.rs` — Split left column into 3 zones (teams, projects, issues)
- `keys.rs` — Handle Focus::TeamList and Focus::ProjectList
- `handlers.rs` — SelectTeam/SelectProject actions with API calls
- `ui.rs` — Wire up new panel renderers

## New Files

- `panels/teams.rs` — Team list box rendering
- `panels/projects.rs` — Project list box rendering

## Color Rules

- Focused box border: `Color::Cyan` with `Modifier::BOLD`
- Unfocused box border: `Color::DarkGray`
- Active item marker: `►` in `Color::Cyan`
- Selected cursor row: `Color::Rgb(30, 35, 50)` background
- Never use `bg(Color::Black)` — always terminal default
