//! Just recipe discovery module.
//!
//! This module provides functionality to discover available Just recipes
//! in the current directory by running `just --list` and parsing its output.

use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use std::process::Command;

/// Discovers available Just recipes in the current directory.
///
/// This function:
/// 1. Checks if `just` is available on PATH
/// 2. Runs `just --list --unsorted` to get all recipes
/// 3. Parses the output to extract recipe names and descriptions
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - `just` is not installed or not on PATH
/// - The justfile doesn't exist or is invalid
/// - `just --list` returns a non-zero exit code
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // First check if just is available
    let just_check = Command::new("just").arg("--version").output();

    match just_check {
        Err(_) => {
            return Err(eyre!(
                "just not found on PATH. Please install just and try again."
            ))
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!(
                "just command failed. Please check your just installation."
            ))
        }
        _ => {}
    }

    // Run just --list --unsorted to get all recipes
    let output = Command::new("just")
        .arg("--list")
        .arg("--unsorted")
        .output()
        .map_err(|e| eyre!("Failed to execute just --list: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "just --list failed: {}",
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_just_list_output(&stdout)
}

/// Parses the output of `just --list` into a list of tasks.
///
/// The format of `just --list` is typically:
/// ```text
/// Available recipes:
///     recipe-name # description
///     another-recipe
/// ```
///
/// This function handles:
/// - Lines with both recipe name and description
/// - Lines with only recipe name
/// - Skips header lines and empty lines
///
/// # Arguments
///
/// * `output` - The stdout from `just --list`
///
/// # Returns
///
/// A vector of discovered tasks with names and optional descriptions.
fn parse_just_list_output(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip the header line "Available recipes:"
        if trimmed.starts_with("Available recipes") {
            continue;
        }

        // Parse recipe line
        // Format is typically: "    recipe-name # description"
        // or just: "    recipe-name"

        // Split by '#' to separate name from description
        let parts: Vec<&str> = trimmed.splitn(2, '#').collect();

        let name = parts[0].trim();

        // Skip if name is empty (shouldn't happen but be defensive)
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
            runner: TaskRunner::Just,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!(
            "No tasks discovered. Is there a justfile in this directory?"
        ));
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_just_list_basic() {
        let output = r#"Available recipes:
    build # Build the project
    test
    deploy # Deploy to production
"#;

        let tasks = parse_just_list_output(output).unwrap();

        assert_eq!(tasks.len(), 3);

        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));

        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[1].description, None);

        assert_eq!(tasks[2].name, "deploy");
        assert_eq!(tasks[2].description, Some("Deploy to production".to_string()));
    }

    #[test]
    fn test_parse_just_list_empty() {
        let output = "Available recipes:\n";

        let result = parse_just_list_output(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_just_list_with_extra_whitespace() {
        let output = r#"Available recipes:

    build   #   Build the project

    test
"#;

        let tasks = parse_just_list_output(output).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));
    }

    #[test]
    fn test_parse_just_list_no_header() {
        let output = r#"    build # Build the project
    test
"#;

        let tasks = parse_just_list_output(output).unwrap();
        assert_eq!(tasks.len(), 2);
    }
}
