/// Core application data structures and state management for Taskpad.

use std::collections::HashMap;
use std::time::SystemTime;

/// Represents a position in the log pane (line index, column index)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogPosition {
    pub line: usize,
    pub col: usize,
}

impl LogPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Text selection state in the log pane
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSelection {
    pub start: LogPosition,
    pub end: LogPosition,
}

impl LogSelection {
    pub fn new(start: LogPosition, end: LogPosition) -> Self {
        Self { start, end }
    }

    /// Returns the selection in normalized order (start before end)
    pub fn normalized(&self) -> (LogPosition, LogPosition) {
        if self.start.line < self.end.line
            || (self.start.line == self.end.line && self.start.col <= self.end.col)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }
}

/// Task runner type.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskRunner {
    /// Just command runner
    Just,
    /// Make build tool
    Make,
}

impl TaskRunner {
    /// Returns the display prefix for this runner
    pub fn prefix(&self) -> &str {
        match self {
            TaskRunner::Just => "[just]",
            TaskRunner::Make => "[make]",
        }
    }

    /// Returns the command name for this runner
    pub fn command(&self) -> &str {
        match self {
            TaskRunner::Just => "just",
            TaskRunner::Make => "make",
        }
    }
}

/// Represents a task that can be executed.
///
/// Supports both Just recipes and Make targets.
#[derive(Debug, Clone)]
pub struct Task {
    /// Stable identifier for the task
    pub id: usize,
    /// User-facing name (recipe/target name)
    pub name: String,
    /// Optional description from task runner output
    pub description: Option<String>,
    /// The task runner that executes this task
    pub runner: TaskRunner,
}

/// Status of a task execution.
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    /// Task is currently running
    Running,
    /// Task completed successfully with the given exit code
    Success(i32),
    /// Task failed with the given exit code
    Failed(i32),
}

impl TaskStatus {
    /// Returns a user-friendly string representation of the status
    pub fn display_string(&self) -> String {
        match self {
            TaskStatus::Running => "Running".to_string(),
            TaskStatus::Success(code) => format!("Success (exit={})", code),
            TaskStatus::Failed(code) => format!("Failed (exit={})", code),
        }
    }
}

/// Represents a task that is currently running or was recently run.
#[derive(Debug)]
pub struct RunningTask {
    /// The task being executed
    pub task: Task,
    /// Current status of the execution
    pub status: TaskStatus,
    /// Log lines (stdout and stderr combined)
    pub log_lines: Vec<String>,
}

impl RunningTask {
    /// Creates a new RunningTask with empty logs and Running status
    pub fn new(task: Task) -> Self {
        Self {
            task,
            status: TaskStatus::Running,
            log_lines: Vec::new(),
        }
    }

    /// Appends a log line to the task's output
    pub fn append_log(&mut self, line: String) {
        self.log_lines.push(line);
    }

    /// Updates the task's status
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// Clears all log lines
    pub fn clear_logs(&mut self) {
        self.log_lines.clear();
    }
}

/// Auto-scroll direction during drag selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragScrollDirection {
    Up,
    Down,
}

/// Represents a task execution entry in history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    /// The task that was executed
    pub task_name: String,
    /// The runner used
    pub runner: TaskRunner,
    /// When the task was executed
    pub timestamp: SystemTime,
    /// Final status of the task
    pub status: TaskStatus,
}

/// Main application state.
///
/// This structure holds all state needed to render the UI and handle events.
/// Rendering should be a pure function of this state.
#[derive(Debug)]
pub struct AppState {
    /// List of all discovered tasks
    pub tasks: Vec<Task>,
    /// Index of the currently selected task in the list
    pub selected_index: usize,
    /// The currently running or last run task (if any)
    pub running_task: Option<RunningTask>,
    /// Log history for each task (keyed by task ID)
    pub task_logs: HashMap<usize, Vec<String>>,
    /// Text selections for each task (keyed by task ID)
    pub task_selections: HashMap<usize, LogSelection>,
    /// Temporary status message for errors, hints, etc.
    pub message: Option<String>,
    /// Flag to indicate the user wants to quit
    pub quitting: bool,
    /// Vertical scroll offset for the task list pane
    pub task_scroll_offset: usize,
    /// Whether to show the info box for the selected task
    pub show_info: bool,
    /// Scroll offset for the info box
    pub info_scroll_offset: usize,
    /// Scroll offset for the history container
    pub history_scroll_offset: usize,
    /// Scroll offset for the log pane (0 = showing latest logs)
    pub log_scroll_offset: usize,
    /// Whether auto-scroll is enabled for logs (disabled when user manually scrolls)
    pub log_auto_scroll: bool,
    /// Whether we're actively selecting text (mouse is down)
    pub is_selecting: bool,
    /// Auto-scroll direction during drag selection (if any)
    pub drag_scroll_direction: Option<DragScrollDirection>,
    /// Last mouse position during drag (for updating selection during auto-scroll)
    pub last_drag_position: Option<LogPosition>,
    /// Whether to show the history container
    pub show_history: bool,
    /// History of executed tasks
    pub task_history: Vec<HistoryEntry>,
}

impl AppState {
    /// Creates a new AppState with the given list of tasks
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            selected_index: 0,
            running_task: None,
            task_logs: HashMap::new(),
            task_selections: HashMap::new(),
            message: None,
            quitting: false,
            task_scroll_offset: 0,
            show_info: false,
            info_scroll_offset: 0,
            history_scroll_offset: 0,
            log_scroll_offset: 0,
            log_auto_scroll: true,
            is_selecting: false,
            drag_scroll_direction: None,
            last_drag_position: None,
            show_history: false,
            task_history: Vec::new(),
        }
    }

    /// Creates an AppState with an error message (used when task discovery fails)
    pub fn with_error(message: String) -> Self {
        Self {
            tasks: Vec::new(),
            selected_index: 0,
            running_task: None,
            task_logs: HashMap::new(),
            task_selections: HashMap::new(),
            message: Some(message),
            quitting: false,
            task_scroll_offset: 0,
            show_info: false,
            info_scroll_offset: 0,
            history_scroll_offset: 0,
            log_scroll_offset: 0,
            log_auto_scroll: true,
            is_selecting: false,
            drag_scroll_direction: None,
            last_drag_position: None,
            show_history: false,
            task_history: Vec::new(),
        }
    }

    /// Returns the currently selected task, if any
    pub fn selected_task(&self) -> Option<&Task> {
        self.tasks.get(self.selected_index)
    }

    /// Moves selection up by one, if not already at the top
    pub fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down by one, if not already at the bottom
    pub fn move_selection_down(&mut self) {
        if self.selected_index < self.tasks.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Returns true if a task is currently running
    pub fn is_task_running(&self) -> bool {
        matches!(
            self.running_task.as_ref().map(|t| &t.status),
            Some(TaskStatus::Running)
        )
    }

    /// Starts running a task
    pub fn start_task(&mut self, task: Task) {
        self.running_task = Some(RunningTask::new(task));
        self.message = None;
    }

    /// Appends a log line to the currently running task
    pub fn append_log(&mut self, line: String) {
        if let Some(ref mut running) = self.running_task {
            // Append to the task-specific log history
            self.task_logs
                .entry(running.task.id)
                .or_insert_with(Vec::new)
                .push(line.clone());
            // Also append to the running task for compatibility
            running.append_log(line);
        }
    }

    /// Updates the status of the currently running task
    pub fn update_task_status(&mut self, status: TaskStatus) {
        // First, extract the data we need for history (if applicable)
        let history_data = if !matches!(status, TaskStatus::Running) {
            self.running_task.as_ref().map(|running| {
                (running.task.name.clone(), running.task.runner.clone())
            })
        } else {
            None
        };

        // Update the running task status
        if let Some(ref mut running) = self.running_task {
            running.set_status(status.clone());
            self.message = Some(status.display_string());
        }

        // Add to history when task completes (success or failure)
        if let Some((task_name, runner)) = history_data {
            self.add_to_history(task_name, runner, status);
        }
    }

    /// Clears all task logs
    pub fn clear_logs(&mut self) {
        self.task_logs.clear();
        if let Some(ref mut running) = self.running_task {
            running.clear_logs();
        }
    }

    /// Gets the logs for the currently selected task
    pub fn selected_task_logs(&self) -> Option<&Vec<String>> {
        if let Some(task) = self.selected_task() {
            self.task_logs.get(&task.id)
        } else {
            None
        }
    }

    /// Reloads tasks from the discovery function
    pub fn reload_tasks(&mut self, new_tasks: Vec<Task>) {
        // Try to preserve the selection by task name
        let selected_name = self.selected_task().map(|t| t.name.clone());

        self.tasks = new_tasks;

        // Try to find the previously selected task by name
        if let Some(name) = selected_name {
            if let Some(pos) = self.tasks.iter().position(|t| t.name == name) {
                self.selected_index = pos;
            } else {
                // If not found, reset to first item
                self.selected_index = 0;
            }
        } else {
            self.selected_index = 0;
        }

        self.message = Some("Tasks reloaded".to_string());
    }

    /// Adjusts scroll offset to ensure the selected item is visible
    pub fn adjust_task_scroll(&mut self, visible_height: usize) {
        if self.selected_index < self.task_scroll_offset {
            self.task_scroll_offset = self.selected_index;
        } else if self.selected_index >= self.task_scroll_offset + visible_height {
            self.task_scroll_offset = self.selected_index - visible_height + 1;
        }
    }

    /// Sets a temporary message
    pub fn set_message(&mut self, message: String) {
        self.message = Some(message);
    }

    /// Marks the app for quitting
    pub fn quit(&mut self) {
        self.quitting = true;
    }

    /// Toggles the info box display
    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }

    /// Toggles the history container display
    pub fn toggle_history(&mut self) {
        self.show_history = !self.show_history;
    }

    /// Adds a task execution to history
    pub fn add_to_history(&mut self, task_name: String, runner: TaskRunner, status: TaskStatus) {
        let entry = HistoryEntry {
            task_name,
            runner,
            timestamp: SystemTime::now(),
            status,
        };
        self.task_history.push(entry);
    }

    /// Scrolls the log view up by the given number of lines
    pub fn scroll_logs_up(&mut self, lines: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_add(lines);
        self.log_auto_scroll = false;
    }

    /// Scrolls the log view down by the given number of lines
    pub fn scroll_logs_down(&mut self, lines: usize) {
        self.log_scroll_offset = self.log_scroll_offset.saturating_sub(lines);
        if self.log_scroll_offset == 0 {
            self.log_auto_scroll = true;
        }
    }

    /// Scrolls to the bottom of logs and re-enables auto-scroll
    pub fn scroll_logs_to_bottom(&mut self) {
        self.log_scroll_offset = 0;
        self.log_auto_scroll = true;
    }

    /// Scrolls the info view up by the given number of lines (see earlier content)
    pub fn scroll_info_up(&mut self, lines: usize) {
        self.info_scroll_offset = self.info_scroll_offset.saturating_sub(lines);
    }

    /// Scrolls the info view down by the given number of lines (see later content)
    pub fn scroll_info_down(&mut self, lines: usize) {
        self.info_scroll_offset = self.info_scroll_offset.saturating_add(lines);
    }

    /// Scrolls the history view up by the given number of lines (see earlier content)
    pub fn scroll_history_up(&mut self, lines: usize) {
        self.history_scroll_offset = self.history_scroll_offset.saturating_sub(lines);
    }

    /// Scrolls the history view down by the given number of lines (see later content)
    pub fn scroll_history_down(&mut self, lines: usize) {
        self.history_scroll_offset = self.history_scroll_offset.saturating_add(lines);
    }

    /// Starts running a task and resets log scrolling for new output
    pub fn start_task_with_scroll_reset(&mut self, task: Task) {
        self.start_task(task);
        self.scroll_logs_to_bottom();
    }

    /// Gets the selection for the currently selected task
    pub fn current_task_selection(&self) -> Option<&LogSelection> {
        let task = self.selected_task()?;
        self.task_selections.get(&task.id)
    }

    /// Starts a text selection at the given position for the current task
    pub fn start_selection(&mut self, pos: LogPosition) {
        if let Some(task) = self.selected_task() {
            let task_id = task.id;
            self.task_selections.insert(task_id, LogSelection::new(pos, pos));
            self.is_selecting = true;
        }
    }

    /// Updates the selection end position (during drag) for the current task
    pub fn update_selection(&mut self, pos: LogPosition) {
        if let Some(task) = self.selected_task() {
            let task_id = task.id;
            if let Some(selection) = self.task_selections.get_mut(&task_id) {
                selection.end = pos;
            }
        }
    }

    /// Ends the selection
    pub fn end_selection(&mut self) {
        self.is_selecting = false;
        self.drag_scroll_direction = None;
        self.last_drag_position = None;
    }

    /// Clears the selection for the current task
    pub fn clear_selection(&mut self) {
        if let Some(task_id) = self.selected_task().map(|t| t.id) {
            self.task_selections.remove(&task_id);
        }
        self.is_selecting = false;
    }

    /// Sets the drag scroll direction and last position
    pub fn set_drag_scroll(&mut self, direction: Option<DragScrollDirection>, position: Option<LogPosition>) {
        self.drag_scroll_direction = direction;
        self.last_drag_position = position;
    }

    /// Performs auto-scroll during drag selection
    pub fn perform_drag_scroll(&mut self) {
        if let Some(direction) = self.drag_scroll_direction {
            match direction {
                DragScrollDirection::Up => {
                    self.scroll_logs_up(1);
                }
                DragScrollDirection::Down => {
                    self.scroll_logs_down(1);
                }
            }

            // Update selection to the last known position after scrolling
            if let Some(pos) = self.last_drag_position {
                self.update_selection(pos);
            }
        }
    }

    /// Gets the selected text from logs for the current task
    pub fn get_selected_text(&self) -> Option<String> {
        let task = self.selected_task()?;
        let selection = self.task_selections.get(&task.id)?;
        let log_lines = self.selected_task_logs()?;

        let (start, end) = selection.normalized();

        if start.line >= log_lines.len() {
            return None;
        }

        let mut result = String::new();

        if start.line == end.line {
            // Single line selection
            if let Some(line) = log_lines.get(start.line) {
                let end_col = end.col.min(line.len());
                let start_col = start.col.min(end_col);
                result.push_str(&line[start_col..end_col]);
            }
        } else {
            // Multi-line selection
            for line_idx in start.line..=end.line.min(log_lines.len().saturating_sub(1)) {
                if let Some(line) = log_lines.get(line_idx) {
                    if line_idx == start.line {
                        // First line: from start.col to end
                        let start_col = start.col.min(line.len());
                        result.push_str(&line[start_col..]);
                    } else if line_idx == end.line {
                        // Last line: from beginning to end.col
                        let end_col = end.col.min(line.len());
                        result.push_str(line.get(..end_col).unwrap_or(line));
                    } else {
                        // Middle lines: entire line
                        result.push_str(line);
                    }

                    // Add newline between lines (except after last line)
                    if line_idx < end.line.min(log_lines.len().saturating_sub(1)) {
                        result.push('\n');
                    }
                }
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_selection_up() {
        let tasks = vec![
            Task { id: 0, name: "task1".to_string(), description: None, runner: TaskRunner::Just },
            Task { id: 1, name: "task2".to_string(), description: None, runner: TaskRunner::Just },
            Task { id: 2, name: "task3".to_string(), description: None, runner: TaskRunner::Just },
        ];
        let mut app = AppState::new(tasks);
        app.selected_index = 1;

        app.move_selection_up();
        assert_eq!(app.selected_index, 0);

        // Should not go below 0
        app.move_selection_up();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_move_selection_down() {
        let tasks = vec![
            Task { id: 0, name: "task1".to_string(), description: None, runner: TaskRunner::Just },
            Task { id: 1, name: "task2".to_string(), description: None, runner: TaskRunner::Just },
            Task { id: 2, name: "task3".to_string(), description: None, runner: TaskRunner::Just },
        ];
        let mut app = AppState::new(tasks);

        app.move_selection_down();
        assert_eq!(app.selected_index, 1);

        app.move_selection_down();
        assert_eq!(app.selected_index, 2);

        // Should not go beyond last item
        app.move_selection_down();
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn test_is_task_running() {
        let mut app = AppState::new(vec![]);
        assert!(!app.is_task_running());

        let task = Task { id: 0, name: "test".to_string(), description: None, runner: TaskRunner::Just };
        app.start_task(task);
        assert!(app.is_task_running());

        app.update_task_status(TaskStatus::Success(0));
        assert!(!app.is_task_running());
    }
}
