//! TYPETUI - A terminal-based typing test application.
//!
//! This is a TUI (Text User Interface) typing test similar to MonkeyType,
//! built with Rust using the ratatui library.
//!
//! ## Features
//!
//! - Two test modes: timed (30s, 60s, 90s, 120s) or word count (10-200 words)
//! - Real-time feedback with color-coded characters
//! - WPM (words per minute) and accuracy statistics
//! - Vim-style keybindings (hjkl navigation)
//!
//! ## Usage
//!
//! Run the application with `cargo run`. Use arrow keys or hjkl to navigate
//! the menu, Enter to start, and Esc to quit.

mod app;
mod ui;
mod words;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppState};

// =============================================================================
// Main Entry Point
// =============================================================================

/// Application entry point.
///
/// Sets up the terminal, runs the main loop, and restores the terminal on exit.
fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal);
    restore_terminal(&mut terminal)?;

    if let Err(err) = result {
        eprintln!("Error: {err}");
    }

    Ok(())
}

// =============================================================================
// Terminal Setup/Teardown
// =============================================================================

/// Initializes the terminal for TUI rendering.
///
/// Enables raw mode, enters alternate screen, and enables mouse capture.
fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

/// Restores the terminal to its original state.
///
/// Disables raw mode, leaves alternate screen, and disables mouse capture.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()
}

// =============================================================================
// Main Loop
// =============================================================================

/// Main application loop.
///
/// Handles rendering and input processing until the user quits.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if handle_input(&mut app, key.code) {
                    return Ok(());
                }
            }
        }

        app.tick();
    }
}

// =============================================================================
// Input Handling
// =============================================================================

/// Routes input to the appropriate handler based on app state.
///
/// Returns `true` if the application should quit.
fn handle_input(app: &mut App, key: KeyCode) -> bool {
    match app.state {
        AppState::Menu => handle_menu_input(app, key),
        AppState::Running => handle_running_input(app, key),
        AppState::Finished => handle_finished_input(app, key),
    }
}

/// Handles input on the menu screen.
///
/// Supports arrow keys and vim-style navigation (hjkl).
fn handle_menu_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        KeyCode::Left | KeyCode::Char('h') => app.menu_left(),
        KeyCode::Right | KeyCode::Char('l') => app.menu_right(),
        KeyCode::Enter => app.start_test(),
        KeyCode::Esc | KeyCode::Char('q') => return true,
        _ => {}
    }
    false
}

/// Handles input during the typing test.
///
/// Space advances to the next word (if complete), backspace deletes,
/// and Esc returns to the menu.
fn handle_running_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Esc => app.reset(),
        KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Tab => app.on_space(),
        KeyCode::Char(c) => app.on_char(c),
        KeyCode::Backspace => app.on_backspace(),
        _ => {}
    }
    false
}

/// Handles input on the results screen.
///
/// 'r' returns to menu, 'q' or Esc quits the application.
fn handle_finished_input(app: &mut App, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('r') => app.reset(),
        KeyCode::Char('q') | KeyCode::Esc => return true,
        _ => {}
    }
    false
}
