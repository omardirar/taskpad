/// UI rendering module.
///
/// This module contains all layout and drawing logic for the TUI.
/// Rendering is a pure function of the AppState.
use crate::app::{AppState, TaskStatus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

/// Layout constants
const TASK_LIST_WIDTH: u16 = 35;

/// Renders the entire application UI.
///
/// This function is called every frame and draws the complete UI
/// based on the current application state.
///
/// # Arguments
///
/// * `frame` - The ratatui Frame to draw on
/// * `app` - The current application state
pub fn render(frame: &mut Frame, app: &AppState) {
    let size = frame.area();

    // Create the main layout: top bar, content area, bottom bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top status bar
            Constraint::Min(0),    // Content area
            Constraint::Length(1), // Bottom key hints bar
        ])
        .split(size);

    // Render top status bar
    render_status_bar(frame, app, chunks[0]);

    // Split the content area into left (tasks) and right (logs)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(TASK_LIST_WIDTH), // Task list
            Constraint::Min(0),                  // Log pane
        ])
        .split(chunks[1]);

    // Render task list
    render_task_list(frame, app, content_chunks[0]);

    // Render log pane
    render_log_pane(frame, app, content_chunks[1]);

    // Render bottom key hints bar
    render_key_hints(frame, chunks[2]);
}

/// Renders the top status bar showing app name and current status.
fn render_status_bar(frame: &mut Frame, app: &AppState, area: Rect) {
    let status_text = if let Some(ref msg) = app.message {
        format!("Taskpad | {}", msg)
    } else if let Some(ref running) = app.running_task {
        match running.status {
            TaskStatus::Running => format!(
                "Taskpad | Running: {} {}",
                running.task.runner.prefix(),
                running.task.name
            ),
            TaskStatus::Success(code) => format!(
                "Taskpad | Last: {} {} (exit={})",
                running.task.runner.prefix(),
                running.task.name,
                code
            ),
            TaskStatus::Failed(code) => format!(
                "Taskpad | Failed: {} {} (exit={})",
                running.task.runner.prefix(),
                running.task.name,
                code
            ),
        }
    } else {
        "Taskpad | Idle".to_string()
    };

    let style = if app.is_task_running() {
        Style::default().fg(Color::Yellow)
    } else if let Some(ref running) = app.running_task {
        match running.status {
            TaskStatus::Success(_) => Style::default().fg(Color::Green),
            TaskStatus::Failed(_) => Style::default().fg(Color::Red),
            _ => Style::default(),
        }
    } else {
        Style::default()
    };

    let status = Paragraph::new(status_text).style(style);
    frame.render_widget(status, area);
}

/// Renders the task list pane on the left side.
fn render_task_list(frame: &mut Frame, app: &AppState, area: Rect) {
    let block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    if app.tasks.is_empty() {
        let message = if app.message.is_some() {
            // Error message is shown in status bar
            Paragraph::new("No tasks available.\nPress 'q' to quit.")
                .block(block)
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new("Loading tasks...")
                .block(block)
                .wrap(Wrap { trim: true })
        };
        frame.render_widget(message, area);
        return;
    }

    // Calculate visible range based on scroll offset
    let inner_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let start = app.task_scroll_offset;
    let end = (start + inner_height).min(app.tasks.len());

    // Create list items for visible tasks
    let items: Vec<ListItem> = app.tasks[start..end]
        .iter()
        .enumerate()
        .map(|(idx, task)| {
            let actual_idx = start + idx;
            let is_selected = actual_idx == app.selected_index;

            // Check if this task is the currently running one
            let is_running = app
                .running_task
                .as_ref()
                .map(|rt| rt.task.name == task.name && rt.status == TaskStatus::Running)
                .unwrap_or(false);

            let prefix = if is_running {
                "▶ "
            } else if is_selected {
                "> "
            } else {
                "  "
            };

            // Create styled line with bold runner prefix
            let mut spans = vec![
                Span::raw(prefix),
                Span::styled(
                    format!("{} ", task.runner.prefix()),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                ),
                Span::raw(&task.name),
            ];

            if let Some(ref desc) = task.description {
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(desc, Style::default().fg(Color::Gray)));
            }

            let line = Line::from(spans);

            let style = if is_selected {
                Style::default().bg(Color::White).fg(Color::Black)
            } else if is_running {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Renders the log pane on the right side showing task output.
fn render_log_pane(frame: &mut Frame, app: &AppState, area: Rect) {
    let title = if let Some(ref running) = app.running_task {
        format!(
            "Logs - {} {}",
            running.task.runner.prefix(),
            running.task.name
        )
    } else {
        "Logs".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    if let Some(ref running) = app.running_task {
        if running.log_lines.is_empty() {
            let message = Paragraph::new("No output yet...")
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(message, area);
            return;
        }

        // For v0, auto-scroll to bottom by showing the last N lines that fit
        let inner_height = area.height.saturating_sub(2) as usize; // Subtract borders
        let total_lines = running.log_lines.len();

        let start = total_lines.saturating_sub(inner_height);

        let visible_lines = &running.log_lines[start..];

        // Convert log lines to Text with appropriate styling
        let lines: Vec<Line> = visible_lines
            .iter()
            .map(|line| {
                // Highlight stderr lines differently
                if line.starts_with("[stderr]") {
                    Line::from(Span::styled(line.clone(), Style::default().fg(Color::Red)))
                } else if line.starts_with("ERROR") {
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("Starting task:") || line.starts_with("Task exited") {
                    Line::from(Span::styled(line.clone(), Style::default().fg(Color::Cyan)))
                } else {
                    Line::from(line.clone())
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    } else {
        let message =
            Paragraph::new("Select a task and press Enter to run it.\nOutput will appear here.")
                .block(block)
                .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
    }
}

/// Renders the bottom key hints bar.
fn render_key_hints(frame: &mut Frame, area: Rect) {
    let hints = vec![
        Span::raw("↑/↓,k/j:"),
        Span::styled(" select ", Style::default().fg(Color::Cyan)),
        Span::raw("│ Enter:"),
        Span::styled(" run ", Style::default().fg(Color::Cyan)),
        Span::raw("│ r:"),
        Span::styled(" reload ", Style::default().fg(Color::Cyan)),
        Span::raw("│ c:"),
        Span::styled(" clear ", Style::default().fg(Color::Cyan)),
        Span::raw("│ q:"),
        Span::styled(" quit", Style::default().fg(Color::Cyan)),
    ];

    let hints_line = Line::from(hints);
    let paragraph = Paragraph::new(hints_line);
    frame.render_widget(paragraph, area);
}
