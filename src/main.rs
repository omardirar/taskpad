//! Taskpad - A keyboard-driven TUI task launcher with first-class support for just recipes.
//!
//! This is the main entry point that sets up the terminal, discovers tasks,
//! and runs the main event loop.

mod app;
mod process;
mod tasks;
mod ui;

use app::{AppState, TaskStatus};
use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind},
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
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q')
        {
            break;
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
        // Calculate the actual visible height for the task list based on layout
        let terminal_height = terminal.size()?.height;
        let content_height = terminal_height.saturating_sub(2) as usize; // Subtract top and bottom bars
        let task_list_outer_height = match (app.show_history, app.show_info) {
            (true, true) => content_height.saturating_sub(8 + 6), // history (8) + info (6)
            (true, false) => content_height.saturating_sub(8),    // history (8)
            (false, true) => content_height.saturating_sub(6),    // info (6)
            (false, false) => content_height,
        };
        let task_list_inner_height = task_list_outer_height.saturating_sub(2); // Subtract borders
        app.adjust_task_scroll(task_list_inner_height);

        // Check for process events (log lines, status updates)
        if let Some(ref rx) = log_rx {
            while let Ok(line) = rx.try_recv() {
                app.append_log(line);
            }
        }

        if let Some(ref rx) = status_rx
            && let Ok(status) = rx.try_recv()
        {
            app.update_task_status(status);
            // Task finished, clear the receivers
            log_rx = None;
            status_rx = None;
        }

        // Handle auto-scroll during drag selection
        if app.is_selecting {
            app.perform_drag_scroll();
        }

        // Poll for keyboard and mouse events with a short timeout
        if event::poll(Duration::from_millis(16))? { // ~60 FPS for smoother streaming
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, key, &mut log_rx, &mut status_rx);
                    }
                }
                Event::Mouse(mouse) => {
                    let terminal_height = terminal.size()?.height;
                    handle_mouse_event(&mut app, mouse, terminal_height, &mut log_rx, &mut status_rx);
                }
                _ => {}
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

/// Determines which region the mouse is over on the left side
#[derive(Debug, PartialEq)]
enum LeftRegion {
    TaskList,
    History,
    Info,
}

/// Calculates which left-side region the mouse is over based on layout
fn get_left_region(app: &AppState, row: u16, terminal_height: u16) -> LeftRegion {
    // Row 0: top bar
    // Row 1: border top
    // Remaining rows split based on what's visible

    if row <= 1 {
        return LeftRegion::TaskList;
    }

    // Calculate the content area height (subtract top bar + bottom bar)
    let content_height = terminal_height.saturating_sub(2);

    match (app.show_history, app.show_info) {
        (true, true) => {
            // Tasks at top, history (8 lines), info (6 lines) at bottom
            let history_height = 8;
            let info_height = 6;
            let task_list_height = content_height.saturating_sub(history_height + info_height);

            let history_start = 1 + task_list_height;
            let info_start = history_start + history_height;

            if row < history_start {
                LeftRegion::TaskList
            } else if row < info_start {
                LeftRegion::History
            } else {
                LeftRegion::Info
            }
        }
        (true, false) => {
            // Tasks at top, history (8 lines) at bottom
            let history_height = 8;
            let history_start = 1 + content_height.saturating_sub(history_height);

            if row < history_start {
                LeftRegion::TaskList
            } else {
                LeftRegion::History
            }
        }
        (false, true) => {
            // Tasks at top, info (6 lines) at bottom
            let info_height = 6;
            let info_start = 1 + content_height.saturating_sub(info_height);

            if row < info_start {
                LeftRegion::TaskList
            } else {
                LeftRegion::Info
            }
        }
        (false, false) => {
            // Only tasks
            LeftRegion::TaskList
        }
    }
}

/// Handles scroll up event
fn handle_scroll_up(app: &mut AppState, column: u16, row: u16, terminal_height: u16) {
    const TASK_LIST_WIDTH: u16 = 35;

    if column >= TASK_LIST_WIDTH {
        // Right side: scroll logs
        app.scroll_logs_up(3);
    } else {
        // Left side: determine which region and scroll accordingly
        match get_left_region(app, row, terminal_height) {
            LeftRegion::TaskList => {
                // Move selection up (scroll follows selection automatically)
                app.move_selection_up();
            }
            LeftRegion::History => {
                app.scroll_history_up(3);
            }
            LeftRegion::Info => {
                app.scroll_info_up(3);
            }
        }
    }
}

/// Handles scroll down event
fn handle_scroll_down(app: &mut AppState, column: u16, row: u16, terminal_height: u16) {
    const TASK_LIST_WIDTH: u16 = 35;

    if column >= TASK_LIST_WIDTH {
        // Right side: scroll logs
        app.scroll_logs_down(3);
    } else {
        // Left side: determine which region and scroll accordingly
        match get_left_region(app, row, terminal_height) {
            LeftRegion::TaskList => {
                // Move selection down (scroll follows selection automatically)
                app.move_selection_down();
            }
            LeftRegion::History => {
                app.scroll_history_down(3);
            }
            LeftRegion::Info => {
                app.scroll_info_down(3);
            }
        }
    }
}

/// Handles mouse input events.
fn handle_mouse_event(
    app: &mut AppState,
    mouse: MouseEvent,
    terminal_height: u16,
    _log_rx: &mut Option<Receiver<String>>,
    _status_rx: &mut Option<Receiver<TaskStatus>>,
) {
    // Task list width from ui module
    const TASK_LIST_WIDTH: u16 = 35;

    match mouse.kind {
        // Handle left click
        MouseEventKind::Down(MouseButton::Left) => {
            // Check if click is within the task list area
            // Task list: x in [0, TASK_LIST_WIDTH), y >= 2 (after top bar and border)
            if mouse.column < TASK_LIST_WIDTH && mouse.row >= 2 {
                // Calculate which task was clicked
                // Subtract 2 for top bar (1) and task list border (1)
                let clicked_row = (mouse.row - 2) as usize;
                let task_index = clicked_row + app.task_scroll_offset;

                // Update selection if valid
                if task_index < app.tasks.len() {
                    app.selected_index = task_index;
                }
            } else if mouse.column >= TASK_LIST_WIDTH && mouse.row >= 2 {
                // Click in logs area - start text selection
                // Convert screen coordinates to log line/column
                if let Some(pos) = screen_to_log_position(app, mouse.column, mouse.row, terminal_height) {
                    app.start_selection(pos);
                }
            }
        }

        // Handle mouse drag for text selection
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.is_selecting && mouse.column >= TASK_LIST_WIDTH && mouse.row >= 2
                && let Some(pos) = screen_to_log_position(app, mouse.column, mouse.row, terminal_height)
            {
                app.update_selection(pos);

                // Check if we should auto-scroll
                // Top edge threshold: within 3 rows of top of logs (row 2 + 3 = 5)
                // Bottom edge threshold: within 3 rows of bottom of terminal (terminal_height - 3)
                const SCROLL_THRESHOLD: u16 = 3;
                let log_top = 2; // Top bar (1) + log border (1)
                let log_bottom = terminal_height.saturating_sub(1); // Bottom bar (1)

                if mouse.row <= log_top + SCROLL_THRESHOLD {
                    // Near top edge - scroll up
                    app.set_drag_scroll(Some(app::DragScrollDirection::Up), Some(pos));
                } else if mouse.row >= log_bottom.saturating_sub(SCROLL_THRESHOLD) {
                    // Near bottom edge - scroll down
                    app.set_drag_scroll(Some(app::DragScrollDirection::Down), Some(pos));
                } else {
                    // Not near edges - stop auto-scrolling
                    app.set_drag_scroll(None, Some(pos));
                }
            }
        }

        // Handle mouse up
        MouseEventKind::Up(MouseButton::Left) => {
            if app.is_selecting {
                app.end_selection();
            }
        }

        // Handle scroll wheel
        MouseEventKind::ScrollUp => {
            handle_scroll_up(app, mouse.column, mouse.row, terminal_height);
        }

        MouseEventKind::ScrollDown => {
            handle_scroll_down(app, mouse.column, mouse.row, terminal_height);
        }

        _ => {}
    }
}

/// Converts screen coordinates to log line and column position
fn screen_to_log_position(app: &AppState, screen_col: u16, screen_row: u16, terminal_height: u16) -> Option<app::LogPosition> {
    use app::LogPosition;

    // Task list width and borders
    const TASK_LIST_WIDTH: u16 = 35;

    // Calculate the log pane inner area
    // Log pane starts at TASK_LIST_WIDTH, with 1 char border on left
    // Top bar (1) + log pane border top (1) = 2
    let log_inner_left = TASK_LIST_WIDTH + 1;
    let log_inner_top = 2;

    if screen_col < log_inner_left || screen_row < log_inner_top {
        return None;
    }

    let col_in_log = (screen_col - log_inner_left) as usize;
    let row_in_visible_area = (screen_row - log_inner_top) as usize;

    // Get the logs for the selected task
    let log_lines = app.selected_task_logs()?;
    if log_lines.is_empty() {
        return None;
    }

    // Calculate actual visible height for logs
    // Terminal height - top bar (1) - bottom bar (1) - log borders (2) = inner height
    let inner_height = terminal_height.saturating_sub(4) as usize;

    // Account for scrolling to find the actual line index
    let total_lines = log_lines.len();

    let visible_start = if app.log_auto_scroll && app.log_scroll_offset == 0 {
        // Auto-scroll mode: show the last N lines
        total_lines.saturating_sub(inner_height)
    } else {
        // Manual scroll mode: calculate from scroll offset
        let max_scroll = total_lines.saturating_sub(inner_height);
        let actual_offset = app.log_scroll_offset.min(max_scroll);
        max_scroll.saturating_sub(actual_offset)
    };

    let line_idx = visible_start + row_in_visible_area;

    if line_idx >= log_lines.len() {
        return None;
    }

    Some(LogPosition::new(line_idx, col_in_log))
}

/// Handles keyboard input events.
fn handle_key_event(
    app: &mut AppState,
    key: KeyEvent,
    log_rx: &mut Option<Receiver<String>>,
    status_rx: &mut Option<Receiver<TaskStatus>>,
) {
    use crossterm::event::KeyModifiers;

    match key.code {
        // Quit
        KeyCode::Char('q') => {
            app.quit();
        }

        // Copy selected text (Ctrl+C or 'y' for yank)
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Some(text) = app.get_selected_text() {
                if let Err(e) = copy_to_clipboard(&text) {
                    app.set_message(format!("Failed to copy: {}", e));
                } else {
                    app.set_message(format!("Copied {} chars to clipboard", text.len()));
                    app.clear_selection();
                }
            }
        }

        KeyCode::Char('y') => {
            if let Some(text) = app.get_selected_text() {
                if let Err(e) = copy_to_clipboard(&text) {
                    app.set_message(format!("Failed to copy: {}", e));
                } else {
                    app.set_message(format!("Copied {} chars to clipboard", text.len()));
                    app.clear_selection();
                }
            }
        }

        // Clear selection (Escape)
        KeyCode::Esc => {
            app.clear_selection();
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

                // Start the task and reset log scrolling
                app.start_task_with_scroll_reset(task.clone());
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
            app.clear_selection();
            app.set_message("Logs cleared".to_string());
        }

        // Toggle info box
        KeyCode::Char('i') => {
            app.toggle_info();
        }

        // Toggle history container
        KeyCode::Char('h') => {
            app.toggle_history();
        }

        // Scroll logs up
        KeyCode::PageUp => {
            app.scroll_logs_up(10);
        }

        // Scroll logs down
        KeyCode::PageDown => {
            app.scroll_logs_down(10);
        }

        // Scroll logs to bottom
        KeyCode::End => {
            app.scroll_logs_to_bottom();
        }

        _ => {}
    }
}

/// Copies text to the system clipboard
fn copy_to_clipboard(text: &str) -> Result<()> {
    use arboard::Clipboard;

    let mut clipboard = Clipboard::new()?;

    // On Linux, use SetExtLinux to persist clipboard contents
    #[cfg(target_os = "linux")]
    {
        use arboard::SetExtLinux;
        clipboard.set()
            .wait()
            .text(text)?;
    }

    // On other platforms, use the standard set_text
    #[cfg(not(target_os = "linux"))]
    {
        clipboard.set_text(text)?;
    }

    Ok(())
}

/// Sets up the terminal for TUI rendering.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its normal state.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
