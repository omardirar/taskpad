/// Core application data structures and state management for Taskpad.
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
    /// Temporary status message for errors, hints, etc.
    pub message: Option<String>,
    /// Flag to indicate the user wants to quit
    pub quitting: bool,
    /// Vertical scroll offset for the task list pane
    pub task_scroll_offset: usize,
}

impl AppState {
    /// Creates a new AppState with the given list of tasks
    pub fn new(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            selected_index: 0,
            running_task: None,
            message: None,
            quitting: false,
            task_scroll_offset: 0,
        }
    }

    /// Creates an AppState with an error message (used when task discovery fails)
    pub fn with_error(message: String) -> Self {
        Self {
            tasks: Vec::new(),
            selected_index: 0,
            running_task: None,
            message: Some(message),
            quitting: false,
            task_scroll_offset: 0,
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
            running.append_log(line);
        }
    }

    /// Updates the status of the currently running task
    pub fn update_task_status(&mut self, status: TaskStatus) {
        if let Some(ref mut running) = self.running_task {
            running.set_status(status.clone());
            self.message = Some(status.display_string());
        }
    }

    /// Clears the logs of the current or last run task
    pub fn clear_logs(&mut self) {
        if let Some(ref mut running) = self.running_task {
            running.clear_logs();
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_selection_up() {
        let tasks = vec![
            Task {
                id: 0,
                name: "task1".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
            Task {
                id: 1,
                name: "task2".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
            Task {
                id: 2,
                name: "task3".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
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
            Task {
                id: 0,
                name: "task1".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
            Task {
                id: 1,
                name: "task2".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
            Task {
                id: 2,
                name: "task3".to_string(),
                description: None,
                runner: TaskRunner::Just,
            },
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

        let task = Task {
            id: 0,
            name: "test".to_string(),
            description: None,
            runner: TaskRunner::Just,
        };
        app.start_task(task);
        assert!(app.is_task_running());

        app.update_task_status(TaskStatus::Success(0));
        assert!(!app.is_task_running());
    }
}
