use super::app::InteractiveApp;
use super::event::{Event, EventHandler};
use crate::config::get_api_key;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub async fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Check API key first
    get_api_key()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = InteractiveApp::new().await?;
    let events = EventHandler::new(100);

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| super::ui::draw(f, &app))?;

        // Handle events
        match events.recv()? {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('r') if app.mode == super::app::AppMode::Normal => {
                        // Refresh issues
                        let _ = app.refresh_issues().await;
                    }
                    KeyCode::Enter if app.mode == super::app::AppMode::Comment => {
                        // Submit comment
                        let _ = app.submit_comment().await;
                    }
                    KeyCode::Enter if app.mode == super::app::AppMode::EditField => {
                        // Submit edit
                        let _ = app.submit_edit().await;
                    }
                    KeyCode::Enter if app.mode == super::app::AppMode::SelectOption => {
                        // Submit selection
                        let _ = app.submit_edit().await;
                    }
                    _ => app.handle_key(key_event.code),
                }
            }
            Event::Tick => {
                // Handle any periodic updates here
            }
        }

        // Check if we should quit
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