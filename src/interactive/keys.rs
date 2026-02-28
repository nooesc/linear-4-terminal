use crossterm::event::{KeyCode, KeyEvent};
use crate::interactive::app::{Focus, Popup, TextInputContext};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    ScrollUp,
    ScrollDown,

    // Focus
    SwitchPanel,
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

    // Popup: text input
    Confirm,
    Cancel,
    TypeChar(char),
    Backspace,
    Delete,
    MoveCursorLeft,
    MoveCursorRight,
    CursorHome,
    CursorEnd,

    // Popup: picker
    PickerUp,
    PickerDown,
    PickerConfirm,
    PickerCancel,
    PickerToggle,

    // Popup: create form
    NextField,
    PrevField,

    // Team/Project selection
    SelectTeam,
    SelectProject,

    // General
    Help,
    Quit,
    DismissNotification,
    ExternalEditor,

    None,
}

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
        Popup::TextInput(ctx) => match key.code {
            KeyCode::Enter => Action::Confirm,
            KeyCode::Esc => Action::Cancel,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Delete => Action::Delete,
            KeyCode::Left => Action::MoveCursorLeft,
            KeyCode::Right => Action::MoveCursorRight,
            KeyCode::Home => Action::CursorHome,
            KeyCode::End => Action::CursorEnd,
            KeyCode::Char('\x05') if *ctx == TextInputContext::EditDescription => Action::ExternalEditor,
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
        Popup::CreateIssue => match key.code {
            KeyCode::Esc => Action::Cancel,
            KeyCode::Tab | KeyCode::Down => Action::NextField,
            KeyCode::BackTab | KeyCode::Up => Action::PrevField,
            KeyCode::Enter => Action::Confirm,
            KeyCode::Char(c) => Action::TypeChar(c),
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Left => Action::MoveCursorLeft,
            KeyCode::Right => Action::MoveCursorRight,
            _ => Action::None,
        },
        Popup::BulkActions => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::PickerDown,
            KeyCode::Char('k') | KeyCode::Up => Action::PickerUp,
            KeyCode::Enter => Action::PickerConfirm,
            KeyCode::Esc | KeyCode::Char('q') => Action::PickerCancel,
            _ => Action::None,
        },
        // All pickers: Status, Priority, Label, Project, Assignee
        _ => match key.code {
            KeyCode::Char('j') | KeyCode::Down => Action::PickerDown,
            KeyCode::Char('k') | KeyCode::Up => Action::PickerUp,
            KeyCode::Enter => Action::PickerConfirm,
            KeyCode::Esc | KeyCode::Char('q') => Action::PickerCancel,
            KeyCode::Char(' ') => Action::PickerToggle,
            _ => Action::None,
        },
    }
}
