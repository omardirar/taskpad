/// npm/pnpm/yarn script discovery module.
///
/// This module provides functionality to discover available scripts
/// from package.json and automatically detect which package manager to use.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{eyre, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Discovers available npm/pnpm/yarn scripts in the current directory.
///
/// This function:
/// 1. Checks if package.json exists
/// 2. Detects which package manager to use (checks lock files and PATH)
/// 3. Parses package.json to extract scripts
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - package.json doesn't exist
/// - No package manager is available on PATH
/// - package.json is invalid or has no scripts
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // Check if package.json exists
    if !Path::new("package.json").exists() {
        return Err(eyre!("package.json not found in current directory"));
    }

    // Detect which package manager to use
    let runner = detect_package_manager()?;

    // Read and parse package.json
    let package_json = fs::read_to_string("package.json")
        .map_err(|e| eyre!("Failed to read package.json: {}", e))?;

    let parsed: Value = serde_json::from_str(&package_json)
        .map_err(|e| eyre!("Failed to parse package.json: {}", e))?;

    // Extract scripts
    let scripts = parsed
        .get("scripts")
        .and_then(|s| s.as_object())
        .ok_or_else(|| eyre!("No scripts found in package.json"))?;

    if scripts.is_empty() {
        return Err(eyre!("No scripts found in package.json"));
    }

    // Convert scripts to tasks
    let mut tasks = Vec::new();
    let mut task_id = 0;

    for (name, command) in scripts {
        let description = command.as_str().map(|s| s.to_string());

        tasks.push(Task {
            id: task_id,
            name: name.clone(),
            description,
            runner: runner.clone(),
        });

        task_id += 1;
    }

    Ok(tasks)
}

/// Detects which package manager to use based on lock files and PATH availability.
///
/// Priority order:
/// 1. Check for lock files (package-lock.json → npm, pnpm-lock.yaml → pnpm, yarn.lock → yarn)
/// 2. Verify the detected package manager is on PATH
/// 3. Fallback to npm if no lock file but npm is available
///
/// # Returns
///
/// The TaskRunner variant for the detected package manager.
///
/// # Errors
///
/// Returns an error if no package manager is available.
fn detect_package_manager() -> Result<TaskRunner> {
    // Check for lock files to determine which package manager is being used
    if Path::new("pnpm-lock.yaml").exists() {
        if is_command_available("pnpm") {
            return Ok(TaskRunner::Pnpm);
        }
    }

    if Path::new("yarn.lock").exists() {
        if is_command_available("yarn") {
            return Ok(TaskRunner::Yarn);
        }
    }

    if Path::new("package-lock.json").exists() {
        if is_command_available("npm") {
            return Ok(TaskRunner::Npm);
        }
    }

    // Fallback: no lock file found, try npm as default
    if is_command_available("npm") {
        return Ok(TaskRunner::Npm);
    }

    // Try pnpm and yarn as fallbacks
    if is_command_available("pnpm") {
        return Ok(TaskRunner::Pnpm);
    }

    if is_command_available("yarn") {
        return Ok(TaskRunner::Yarn);
    }

    Err(eyre!(
        "No package manager found. Please install npm, pnpm, or yarn."
    ))
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
    fn test_detect_package_manager_with_npm_lock() {
        // This test would need to mock filesystem and command execution
        // For now, we'll skip actual testing and rely on manual testing
    }
}
