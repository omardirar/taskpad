/// Ruby Rake task discovery module.
///
/// This module provides functionality to discover available Rake tasks
/// from a Rakefile.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use std::process::Command;

/// Discovers available Rake tasks in the current directory.
///
/// This function:
/// 1. Checks if `rake` is available on PATH
/// 2. Runs `rake --tasks` to get all tasks
/// 3. Parses the output to extract task names and descriptions
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - `rake` is not installed or not on PATH
/// - No Rakefile exists
/// - `rake --tasks` returns a non-zero exit code
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // First check if rake is available
    let rake_check = Command::new("rake").arg("--version").output();

    match rake_check {
        Err(_) => {
            return Err(eyre!(
                "rake not found on PATH. Install with: gem install rake"
            ));
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!("rake command failed. Please check your installation."));
        }
        _ => {}
    }

    // Run rake --tasks to get all tasks
    let output = Command::new("rake")
        .arg("--tasks")
        .output()
        .map_err(|e| eyre!("Failed to execute rake --tasks: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("rake --tasks failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_rake_tasks_output(&stdout)
}

/// Parses the output of `rake --tasks` into a list of tasks.
///
/// The format of `rake --tasks` is typically:
/// ```text
/// rake task:name         # Description of the task
/// rake another:task      # Another description
/// rake simple
/// ```
///
/// Tasks start with "rake " followed by the task name, optionally followed by " # " and description.
fn parse_rake_tasks_output(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip lines that don't start with "rake "
        if !trimmed.starts_with("rake ") {
            continue;
        }

        // Remove "rake " prefix
        let without_prefix = &trimmed[5..];

        // Parse task line
        // Format is typically: "task:name  # Description"
        // Split by '#' to separate name from description
        let parts: Vec<&str> = without_prefix.splitn(2, '#').collect();

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
            runner: TaskRunner::Rake,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!(
            "No Rake tasks discovered. Is there a Rakefile in this directory?"
        ));
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rake_tasks_basic() {
        let output = r#"rake build         # Build the project
rake test          # Run tests
rake deploy        # Deploy to production
rake clean
"#;

        let tasks = parse_rake_tasks_output(output).unwrap();

        assert_eq!(tasks.len(), 4);

        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));

        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[1].description, Some("Run tests".to_string()));

        assert_eq!(tasks[2].name, "deploy");
        assert_eq!(
            tasks[2].description,
            Some("Deploy to production".to_string())
        );

        assert_eq!(tasks[3].name, "clean");
        assert_eq!(tasks[3].description, None);
    }

    #[test]
    fn test_parse_rake_tasks_with_namespaces() {
        let output = r#"rake db:migrate    # Run database migrations
rake db:seed       # Seed the database
"#;

        let tasks = parse_rake_tasks_output(output).unwrap();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "db:migrate");
        assert_eq!(
            tasks[0].description,
            Some("Run database migrations".to_string())
        );
    }

    #[test]
    fn test_parse_rake_tasks_empty() {
        let output = "";

        let result = parse_rake_tasks_output(output);
        assert!(result.is_err());
    }
}
