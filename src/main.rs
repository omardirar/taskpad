/// Taskpad - A keyboard-driven TUI task launcher with first-class support for just recipes.
///
/// This is the main entry point that sets up the terminal, discovers tasks,
/// and runs the main event loop.

mod app;
mod process;
mod tasks;
mod ui;

use app::{AppState, TaskStatus};
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Main entry point for Taskpad.
fn main() -> Result<()> {
    // Set up better panic handler
    color_eyre::install()?;

    // Discover tasks from all available sources (Just and Make)
    let tasks = match tasks::discover_all_tasks() {
        Ok(tasks) => tasks,
        Err(e) => {
            // If discovery fails, create an AppState with the error
            // and let the user see it in the TUI before quitting
            let app = AppState::with_error(e.to_string());
            return run_app_with_error(app);
        }
    };

    // Create initial app state
    let app = AppState::new(tasks);

    // Run the main application
    run_app(app)
}

/// Runs the app when there's an error discovering tasks.
/// Shows the error in the TUI and waits for the user to quit.
fn run_app_with_error(app: AppState) -> Result<()> {
    // Set up terminal
    let mut terminal = setup_terminal()?;

    // Draw the UI once to show the error
    terminal.draw(|frame| ui::render(frame, &app))?;

    // Wait for 'q' key to quit
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;

    Ok(())
}

/// Runs the main application with the given initial state.
fn run_app(mut app: AppState) -> Result<()> {
    // Set up terminal
    let mut terminal = setup_terminal()?;

    // Create channels for process communication
    // These will be created fresh each time we start a task
    let mut log_rx: Option<Receiver<String>> = None;
    let mut status_rx: Option<Receiver<TaskStatus>> = None;

    // Main event loop
    loop {
        // Render the UI
        terminal.draw(|frame| ui::render(frame, &app))?;

        // Update scroll offset for task list
        let visible_height = terminal.size()?.height.saturating_sub(4) as usize; // Account for borders and bars
        app.adjust_task_scroll(visible_height);

        // Check for process events (log lines, status updates)
        if let Some(ref rx) = log_rx {
            while let Ok(line) = rx.try_recv() {
                app.append_log(line);
            }
        }

        if let Some(ref rx) = status_rx {
            if let Ok(status) = rx.try_recv() {
                app.update_task_status(status);
                // Task finished, clear the receivers
                log_rx = None;
                status_rx = None;
            }
        }

        // Poll for keyboard events with a short timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(&mut app, key, &mut log_rx, &mut status_rx);
                }
            }
        }

        // Check if we should quit
        if app.quitting {
            break;
        }
    }

    // Restore terminal
    restore_terminal(&mut terminal)?;

    Ok(())
}

/// Handles keyboard input events.
fn handle_key_event(
    app: &mut AppState,
    key: KeyEvent,
    log_rx: &mut Option<Receiver<String>>,
    status_rx: &mut Option<Receiver<TaskStatus>>,
) {
    match key.code {
        // Quit
        KeyCode::Char('q') => {
            app.quit();
        }

        // Move selection up
        KeyCode::Up | KeyCode::Char('k') => {
            app.move_selection_up();
        }

        // Move selection down
        KeyCode::Down | KeyCode::Char('j') => {
            app.move_selection_down();
        }

        // Run selected task
        KeyCode::Enter => {
            if app.is_task_running() {
                app.set_message("A task is already running. Wait for it to finish.".to_string());
            } else if let Some(task) = app.selected_task().cloned() {
                // Create new channels for this task
                let (log_tx, new_log_rx) = channel();
                let (status_tx, new_status_rx) = channel();

                *log_rx = Some(new_log_rx);
                *status_rx = Some(new_status_rx);

                // Start the task
                app.start_task(task.clone());
                process::run_task(task, log_tx, status_tx);
            }
        }

        // Reload tasks
        KeyCode::Char('r') => {
            if app.is_task_running() {
                app.set_message("Cannot reload tasks while a task is running.".to_string());
            } else {
                match tasks::discover_all_tasks() {
                    Ok(new_tasks) => {
                        app.reload_tasks(new_tasks);
                    }
                    Err(e) => {
                        app.set_message(format!("Failed to reload tasks: {}", e));
                    }
                }
            }
        }

        // Clear logs
        KeyCode::Char('c') => {
            app.clear_logs();
            app.set_message("Logs cleared".to_string());
        }

        _ => {}
    }
}

/// Sets up the terminal for TUI rendering.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its normal state.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
