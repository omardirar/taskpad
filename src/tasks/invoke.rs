/// Python Invoke task discovery module.
///
/// This module provides functionality to discover available Invoke tasks
/// from a tasks.py or tasks/ directory.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use std::process::Command;

/// Discovers available Invoke tasks in the current directory.
///
/// This function:
/// 1. Checks if `invoke` is available on PATH
/// 2. Runs `invoke --list` to get all tasks
/// 3. Parses the output to extract task names and descriptions
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - `invoke` is not installed or not on PATH
/// - No tasks.py or tasks/ directory exists
/// - `invoke --list` returns a non-zero exit code
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // First check if invoke is available
    let invoke_check = Command::new("invoke").arg("--version").output();

    match invoke_check {
        Err(_) => {
            return Err(eyre!(
                "invoke not found on PATH. Install with: pip install invoke"
            ));
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!("invoke command failed. Please check your installation."));
        }
        _ => {}
    }

    // Run invoke --list to get all tasks
    let output = Command::new("invoke")
        .arg("--list")
        .output()
        .map_err(|e| eyre!("Failed to execute invoke --list: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("invoke --list failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_invoke_list_output(&stdout)
}

/// Parses the output of `invoke --list` into a list of tasks.
///
/// The format of `invoke --list` is typically:
/// ```text
/// Available tasks:
///
///   task-name         Description of the task
///   another-task      Another description
/// ```
///
/// Tasks are indented and descriptions are separated by whitespace.
fn parse_invoke_list_output(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;
    let mut in_task_section = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Look for "Available tasks:" header
        if trimmed.starts_with("Available tasks") {
            in_task_section = true;
            continue;
        }

        // Skip until we're in the task section
        if !in_task_section {
            continue;
        }

        // Tasks are indented with spaces
        if !line.starts_with("  ") {
            continue;
        }

        // Parse task line
        // Format is typically: "  task-name         Description"
        // Split by multiple spaces to separate name from description
        let parts: Vec<&str> = trimmed.splitn(2, "  ").collect();

        let name = parts[0].trim();

        // Skip if name is empty
        if name.is_empty() {
            continue;
        }

        let description = if parts.len() > 1 {
            let desc = parts[1].trim();
            if desc.is_empty() {
                None
            } else {
                Some(desc.to_string())
            }
        } else {
            None
        };

        tasks.push(Task {
            id: task_id,
            name: name.to_string(),
            description,
            runner: TaskRunner::Invoke,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!(
            "No Invoke tasks discovered. Is there a tasks.py or tasks/ directory?"
        ));
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invoke_list_basic() {
        let output = r#"Available tasks:

  build         Build the project
  test          Run tests
  deploy        Deploy to production
"#;

        let tasks = parse_invoke_list_output(output).unwrap();

        assert_eq!(tasks.len(), 3);

        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));

        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[1].description, Some("Run tests".to_string()));

        assert_eq!(tasks[2].name, "deploy");
        assert_eq!(
            tasks[2].description,
            Some("Deploy to production".to_string())
        );
    }

    #[test]
    fn test_parse_invoke_list_no_description() {
        let output = r#"Available tasks:

  build
  test
"#;

        let tasks = parse_invoke_list_output(output).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, None);
    }

    #[test]
    fn test_parse_invoke_list_empty() {
        let output = "Available tasks:\n";

        let result = parse_invoke_list_output(output);
        assert!(result.is_err());
    }
}
