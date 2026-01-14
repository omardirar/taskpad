/// UI rendering module.
///
/// This module contains all layout and drawing logic for the TUI.
/// Rendering is a pure function of the AppState.
use crate::app::{
    AppState, FocusedPane, HistoryEntry, TaskStatus, display_col_to_byte_idx, str_display_width,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};
use std::time::SystemTime;

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

    // Render task list and optional history box on the left
    if app.show_history {
        // History box visible
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Task list
                Constraint::Length(8), // History box (fixed height)
            ])
            .split(content_chunks[0]);

        render_task_list(frame, app, left_chunks[0]);
        render_history_container(frame, app, left_chunks[1]);
    } else {
        // Just task list
        render_task_list(frame, app, content_chunks[0]);
    }

    // Render info box and log pane on the right
    if app.show_info {
        // Info box visible on top of log pane
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Info box (fixed height)
                Constraint::Min(0),    // Log pane
            ])
            .split(content_chunks[1]);

        render_info_box(frame, app, right_chunks[0]);
        render_log_pane(frame, app, right_chunks[1]);
    } else {
        // Just log pane
        render_log_pane(frame, app, content_chunks[1]);
    }

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
    let border_color = if app.focused_pane == FocusedPane::Tasks {
        Color::Cyan
    } else {
        Color::White
    };

    let block = Block::default()
        .title("Tasks")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

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
            let is_selected =
                actual_idx == app.selected_index && app.focused_pane == FocusedPane::Tasks;

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
            let spans = vec![
                Span::raw(prefix),
                Span::styled(
                    format!("{} ", task.runner.prefix()),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                ),
                Span::raw(&task.name),
            ];

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

    // Render scrollbar if there are more tasks than can fit
    let total_tasks = app.tasks.len();
    if total_tasks > inner_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(total_tasks.saturating_sub(inner_height))
            .position(app.task_scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Renders the info box showing the selected task's description.
fn render_info_box(frame: &mut Frame, app: &AppState, area: Rect) {
    let block = Block::default()
        .title("Info")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    if let Some(task) = app.selected_task() {
        let content = if let Some(ref desc) = task.description {
            desc.clone()
        } else {
            "No description available.".to_string()
        };

        // Split content into wrapped lines based on available width
        let inner_width = area.width.saturating_sub(2) as usize; // Subtract borders
        let inner_height = area.height.saturating_sub(2) as usize;

        let mut wrapped_lines = Vec::new();
        for line in content.lines() {
            if str_display_width(line) <= inner_width {
                wrapped_lines.push(line.to_string());
            } else {
                // Wrap long lines with word-awareness using character-safe slicing
                let mut remaining = line;
                while !remaining.is_empty() {
                    if str_display_width(remaining) <= inner_width {
                        wrapped_lines.push(remaining.to_string());
                        break;
                    } else {
                        // Find byte index where display width reaches inner_width
                        let max_byte_idx = display_col_to_byte_idx(remaining, inner_width);
                        // Search backward from that point for whitespace to break at
                        let split_at = remaining[..max_byte_idx]
                            .char_indices()
                            .filter(|(_, ch)| ch.is_whitespace())
                            .next_back()
                            .map(|(pos, ch)| pos + ch.len_utf8()) // Move past the whitespace char
                            .unwrap_or(max_byte_idx); // Fall back to hard split if no whitespace
                        wrapped_lines.push(remaining[..split_at].trim_end().to_string());
                        remaining = remaining[split_at..].trim_start();
                    }
                }
            }
        }

        let total_lines = wrapped_lines.len();

        // Calculate visible range based on scroll offset
        let start = app.info_scroll_offset.min(total_lines.saturating_sub(1));
        let end = (start + inner_height).min(total_lines);
        let visible_lines = &wrapped_lines[start..end];

        let paragraph = Paragraph::new(visible_lines.join("\n"))
            .block(block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, area);

        // Render scrollbar if there are more lines than can fit
        if total_lines > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let mut scrollbar_state = ScrollbarState::new(total_lines.saturating_sub(inner_height))
                .position(app.info_scroll_offset);

            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    } else {
        let message = Paragraph::new("No task selected.")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
    }
}

/// Renders the history container showing recently executed tasks with timestamps.
fn render_history_container(frame: &mut Frame, app: &AppState, area: Rect) {
    let border_color = if app.focused_pane == FocusedPane::History {
        Color::Cyan
    } else {
        Color::White
    };

    let block = Block::default()
        .title("History")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if app.task_history.is_empty() {
        let message = Paragraph::new("No tasks executed yet.")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(message, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize; // Subtract borders
    let total_entries = app.task_history.len();

    // Create a reversed vec of history (most recent first)
    let reversed_history: Vec<&HistoryEntry> = app.task_history.iter().rev().collect();

    // Calculate visible range based on scroll offset
    let start = app
        .history_scroll_offset
        .min(total_entries.saturating_sub(1));
    let end = (start + inner_height).min(total_entries);
    let visible_entries = &reversed_history[start..end];

    let items: Vec<ListItem> = visible_entries
        .iter()
        .enumerate()
        .map(|(visible_idx, entry)| {
            // Calculate actual index in reversed history
            let actual_idx = start + visible_idx;
            let is_selected = app.selected_history_index == Some(actual_idx)
                && app.focused_pane == FocusedPane::History;

            // Format timestamp
            let timestamp_str = format_timestamp(&entry.timestamp);

            // Format status with color
            let status_span = match entry.status {
                TaskStatus::Success(_) => Span::styled("✓", Style::default().fg(Color::Green)),
                TaskStatus::Failed(_) => Span::styled("✗", Style::default().fg(Color::Red)),
                TaskStatus::Running => Span::styled("⋯", Style::default().fg(Color::Yellow)),
            };

            // Create the line with timestamp, status, runner, and task name
            let prefix = if is_selected { "> " } else { "  " };

            let spans = vec![
                Span::raw(prefix),
                Span::styled(
                    format!("{} ", timestamp_str),
                    Style::default().fg(Color::DarkGray),
                ),
                status_span,
                Span::raw(" "),
                Span::styled(
                    format!("{} ", entry.runner.prefix()),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                ),
                Span::raw(&entry.task_name),
            ];

            let line = Line::from(spans);

            // Apply selection highlighting (inverted colors like task list)
            let style = if is_selected {
                Style::default().bg(Color::White).fg(Color::Black)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);

    // Render scrollbar if there are more entries than can fit
    if total_entries > inner_height {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(total_entries.saturating_sub(inner_height))
            .position(app.history_scroll_offset);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

/// Formats a SystemTime as a human-readable timestamp in local time
fn format_timestamp(time: &SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs() as libc::time_t;
            let mut tm: libc::tm = unsafe { std::mem::zeroed() };
            // SAFETY: localtime_r is thread-safe and writes to our stack-allocated tm
            let result = unsafe { libc::localtime_r(&secs, &mut tm) };
            if result.is_null() {
                return "??:??:??".to_string();
            }
            format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
        }
        Err(_) => "??:??:??".to_string(),
    }
}

/// Renders the log pane on the right side showing task output.
fn render_log_pane(frame: &mut Frame, app: &AppState, area: Rect) {
    let title = if app.is_history_focused() {
        if let Some(entry) = app.selected_history_entry() {
            let timestamp_str = format_timestamp(&entry.timestamp);
            format!(
                "Logs (History) - {} {} - {}",
                entry.runner.prefix(),
                entry.task_name,
                timestamp_str
            )
        } else {
            "Logs (History)".to_string()
        }
    } else if let Some(task) = app.selected_task() {
        format!("Logs - {} {}", task.runner.prefix(), task.name)
    } else {
        "Logs".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    // Get logs based on focus: history logs if history focused, otherwise current task logs
    let log_lines = if app.is_history_focused() {
        app.get_history_logs()
    } else {
        app.selected_task_logs()
    };

    if let Some(log_lines) = log_lines {
        if log_lines.is_empty() {
            let message = Paragraph::new("No output yet...")
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(message, area);
            return;
        }

        let inner_height = area.height.saturating_sub(2) as usize; // Subtract borders
        let total_lines = log_lines.len();

        // Calculate visible range based on scroll offset
        let start = if app.log_auto_scroll && app.log_scroll_offset == 0 {
            // Auto-scroll mode: show the last N lines
            total_lines.saturating_sub(inner_height)
        } else {
            // Manual scroll mode: calculate from scroll offset
            // scroll_offset of 0 means showing the bottom
            // scroll_offset increases as we scroll up
            let max_scroll = total_lines.saturating_sub(inner_height);
            let actual_offset = app.log_scroll_offset.min(max_scroll);
            max_scroll.saturating_sub(actual_offset)
        };

        let end = (start + inner_height).min(total_lines);
        let visible_lines = &log_lines[start..end];

        // Convert log lines to Text with appropriate styling and selection highlighting
        let lines: Vec<Line> = visible_lines
            .iter()
            .enumerate()
            .map(|(visible_idx, line)| {
                let actual_line_idx = start + visible_idx;

                // Get base style for the line
                let base_style = if line.starts_with("[stderr]") {
                    Style::default().fg(Color::Red)
                } else if line.starts_with("ERROR") {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else if line.starts_with("Starting task:") || line.starts_with("Task exited") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };

                // Check if this line has any selection for the current task
                if let Some(selection) = app.current_task_selection() {
                    let (sel_start, sel_end) = selection.normalized();

                    // Check if this line is within the selection range
                    if actual_line_idx >= sel_start.line && actual_line_idx <= sel_end.line {
                        // Build spans with selection highlighting
                        // Convert display columns to byte indices for safe UTF-8 slicing
                        let mut spans = Vec::new();
                        let line_len = line.len();

                        if actual_line_idx == sel_start.line && actual_line_idx == sel_end.line {
                            // Single line selection
                            let start_byte = display_col_to_byte_idx(line, sel_start.col);
                            let end_byte = display_col_to_byte_idx(line, sel_end.col);

                            if start_byte > 0 {
                                spans
                                    .push(Span::styled(line[..start_byte].to_string(), base_style));
                            }
                            if end_byte > start_byte {
                                spans.push(Span::styled(
                                    line[start_byte..end_byte].to_string(),
                                    base_style.bg(Color::DarkGray),
                                ));
                            }
                            if end_byte < line_len {
                                spans.push(Span::styled(line[end_byte..].to_string(), base_style));
                            }
                        } else if actual_line_idx == sel_start.line {
                            // First line of multi-line selection
                            let start_byte = display_col_to_byte_idx(line, sel_start.col);
                            if start_byte > 0 {
                                spans
                                    .push(Span::styled(line[..start_byte].to_string(), base_style));
                            }
                            spans.push(Span::styled(
                                line[start_byte..].to_string(),
                                base_style.bg(Color::DarkGray),
                            ));
                        } else if actual_line_idx == sel_end.line {
                            // Last line of multi-line selection
                            let end_byte = display_col_to_byte_idx(line, sel_end.col);
                            if end_byte > 0 {
                                spans.push(Span::styled(
                                    line[..end_byte].to_string(),
                                    base_style.bg(Color::DarkGray),
                                ));
                            }
                            if end_byte < line_len {
                                spans.push(Span::styled(line[end_byte..].to_string(), base_style));
                            }
                        } else {
                            // Middle line - entire line is selected
                            spans.push(Span::styled(line.clone(), base_style.bg(Color::DarkGray)));
                        }

                        return Line::from(spans);
                    }
                }

                // No selection on this line, use regular styling
                Line::from(Span::styled(line.clone(), base_style))
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);

        // Render scrollbar if there are more lines than can fit
        if total_lines > inner_height {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let max_scroll = total_lines.saturating_sub(inner_height);
            let scroll_position = max_scroll.saturating_sub(app.log_scroll_offset.min(max_scroll));

            let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll_position);

            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
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
        Span::raw("│ ←/→:"),
        Span::styled(" focus ", Style::default().fg(Color::Cyan)),
        Span::raw("│ Enter:"),
        Span::styled(" run ", Style::default().fg(Color::Cyan)),
        Span::raw("│ y/Ctrl+C:"),
        Span::styled(" copy ", Style::default().fg(Color::Cyan)),
        Span::raw("│ h:"),
        Span::styled(" history ", Style::default().fg(Color::Cyan)),
        Span::raw("│ i:"),
        Span::styled(" info ", Style::default().fg(Color::Cyan)),
        Span::raw("│ c:"),
        Span::styled(" clear ", Style::default().fg(Color::Cyan)),
        Span::raw("│ q:"),
        Span::styled(" quit", Style::default().fg(Color::Cyan)),
    ];

    let hints_line = Line::from(hints);
    let paragraph = Paragraph::new(hints_line);
    frame.render_widget(paragraph, area);
}
