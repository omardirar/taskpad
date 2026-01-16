/// Cargo task discovery module.
///
/// This module provides functionality to discover both standard cargo commands
/// and custom cargo-make tasks.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use std::path::Path;
use std::process::Command;

/// Discovers available Cargo tasks.
///
/// This function discovers:
/// 1. Standard cargo commands (build, test, run, etc.) if cargo is available
/// 2. cargo-make custom tasks if Makefile.toml exists and cargo-make is installed
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if cargo is not available.
pub fn discover_tasks() -> Result<Vec<Task>> {
    let mut all_tasks = Vec::new();
    let mut task_id = 0;

    // Check if cargo is available
    if !is_command_available("cargo") {
        return Err(eyre!("cargo not found on PATH"));
    }

    // Add standard cargo commands
    let standard_commands = vec![
        ("build", "Compile the current package"),
        ("test", "Run the tests"),
        ("run", "Run the binary"),
        ("check", "Check compilation without building"),
        ("clippy", "Run the linter"),
        ("fmt", "Format the code"),
        ("doc", "Build documentation"),
        ("bench", "Run benchmarks"),
        ("clean", "Remove build artifacts"),
    ];

    for (name, description) in standard_commands {
        all_tasks.push(Task {
            id: task_id,
            name: name.to_string(),
            description: Some(description.to_string()),
            runner: TaskRunner::Cargo,
        });
        task_id += 1;
    }

    // Try to discover cargo-make tasks
    if let Ok(mut cargo_make_tasks) = discover_cargo_make_tasks() {
        for task in cargo_make_tasks.iter_mut() {
            task.id = task_id;
            task_id += 1;
        }
        all_tasks.extend(cargo_make_tasks);
    }

    if all_tasks.is_empty() {
        return Err(eyre!("No Cargo tasks discovered"));
    }

    Ok(all_tasks)
}

/// Discovers cargo-make tasks from Makefile.toml.
///
/// Returns tasks if:
/// - Makefile.toml exists
/// - cargo-make is installed
/// - Tasks can be listed successfully
fn discover_cargo_make_tasks() -> Result<Vec<Task>> {
    // Check if Makefile.toml exists
    if !Path::new("Makefile.toml").exists() {
        return Err(eyre!("Makefile.toml not found"));
    }

    // Check if cargo-make is available
    let cargo_make_check = Command::new("cargo")
        .arg("make")
        .arg("--version")
        .output();

    match cargo_make_check {
        Err(_) => {
            return Err(eyre!("cargo-make not found. Install with: cargo install cargo-make"));
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!("cargo-make not available"));
        }
        _ => {}
    }

    // Run cargo make --list-all-steps to get all tasks
    let output = Command::new("cargo")
        .arg("make")
        .arg("--list-all-steps")
        .output()
        .map_err(|e| eyre!("Failed to execute cargo make --list-all-steps: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("cargo make --list-all-steps failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_cargo_make_output(&stdout)
}

/// Parses the output of `cargo make --list-all-steps`.
///
/// The format is typically:
/// ```text
/// task-name - description
/// another-task
/// ```
fn parse_cargo_make_output(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;

    for line in output.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip header or separator lines
        if trimmed.starts_with("---") || trimmed.starts_with("===") {
            continue;
        }

        // Parse task line
        // Format can be: "task-name - description" or just "task-name"
        let parts: Vec<&str> = trimmed.splitn(2, " - ").collect();

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
            runner: TaskRunner::CargoMake,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!("No cargo-make tasks discovered from Makefile.toml"));
    }

    Ok(tasks)
}

/// Checks if a command is available on PATH.
fn is_command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_make_output() {
        let output = r#"build - Build the project
test - Run tests
deploy
"#;

        let tasks = parse_cargo_make_output(output).unwrap();

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[0].description, Some("Build the project".to_string()));
        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[2].name, "deploy");
        assert_eq!(tasks[2].description, None);
    }
}
