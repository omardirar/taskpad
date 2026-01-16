/// Python Poe the Poet task discovery module.
///
/// This module provides functionality to discover available Poe tasks
/// from pyproject.toml.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use std::process::Command;

/// Discovers available Poe the Poet tasks in the current directory.
///
/// This function:
/// 1. Checks if `poe` is available on PATH
/// 2. Runs `poe` (without args) to get all tasks
/// 3. Parses the output to extract task names and descriptions
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - `poe` is not installed or not on PATH
/// - No pyproject.toml with [tool.poe.tasks] exists
/// - `poe` returns a non-zero exit code
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // First check if poe is available
    let poe_check = Command::new("poe").arg("--version").output();

    match poe_check {
        Err(_) => {
            return Err(eyre!(
                "poe not found on PATH. Install with: pip install poethepoet"
            ));
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!("poe command failed. Please check your installation."));
        }
        _ => {}
    }

    // Run poe (without args) to get all tasks
    // Note: poe without args shows help with task list
    let output = Command::new("poe")
        .output()
        .map_err(|e| eyre!("Failed to execute poe: {}", e))?;

    // poe returns non-zero when showing help, so we check stdout instead
    let stdout = String::from_utf8_lossy(&output.stdout);

    // If stdout is empty, there might be an error
    if stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            return Err(eyre!("poe failed: {}", stderr.trim()));
        }
        return Err(eyre!("No output from poe command"));
    }

    parse_poe_output(&stdout)
}

/// Parses the output of `poe` (without arguments) into a list of tasks.
///
/// The format of `poe` output is typically:
/// ```text
/// Poe the Poet - A task runner for Python
///
/// USAGE
///   poe [-h] task [task arguments]
///
/// TASKS
///   build         Build the project
///   test          Run tests
///   deploy        Deploy application
/// ```
///
/// Tasks are listed under the TASKS section with descriptions.
fn parse_poe_output(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;
    let mut in_task_section = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Look for "TASKS" section header
        if trimmed.starts_with("TASKS") || trimmed == "tasks" {
            in_task_section = true;
            continue;
        }

        // If we hit another section header (all caps), we're done with tasks
        if in_task_section
            && !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace())
        {
            break;
        }

        // Skip until we're in the task section
        if !in_task_section {
            continue;
        }

        // Skip empty lines
        if trimmed.is_empty() {
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

        // Skip if name is empty or looks like a continuation line
        if name.is_empty() || name.starts_with('-') {
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
            runner: TaskRunner::Poe,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!(
            "No Poe tasks discovered. Is there a pyproject.toml with [tool.poe.tasks]?"
        ));
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_poe_output_basic() {
        let output = r#"Poe the Poet - A task runner for Python

USAGE
  poe [-h] task [task arguments]

TASKS
  build         Build the project
  test          Run tests
  deploy        Deploy application

OPTIONS
  -h, --help    Show this help message
"#;

        let tasks = parse_poe_output(output).unwrap();

        assert_eq!(tasks.len(), 3);

        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));

        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[1].description, Some("Run tests".to_string()));

        assert_eq!(tasks[2].name, "deploy");
        assert_eq!(tasks[2].description, Some("Deploy application".to_string()));
    }

    #[test]
    fn test_parse_poe_output_no_description() {
        let output = r#"TASKS
  build
  test
"#;

        let tasks = parse_poe_output(output).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, None);
    }

    #[test]
    fn test_parse_poe_output_empty() {
        let output = "TASKS\n";

        let result = parse_poe_output(output);
        assert!(result.is_err());
    }
}
