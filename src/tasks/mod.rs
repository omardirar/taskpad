//! Task discovery modules.
//!
//! This module provides functionality for discovering tasks from various sources.
//! Supports both Just recipes and Make targets.

use crate::app::Task;
use color_eyre::eyre::Result;

pub mod just;
pub mod make;

/// Discovers tasks from all available sources (Just and Make).
///
/// This function attempts to discover tasks from:
/// 1. Just recipes (if justfile exists)
/// 2. Make targets (if Makefile exists)
///
/// Tasks from both sources are combined into a single list with unique IDs.
/// If both sources are available, tasks are prefixed with [just] or [make]
/// in the UI (handled by the TaskRunner in the Task struct).
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with all discovered tasks from available sources.
/// Returns an error only if no tasks could be discovered from any source.
pub fn discover_all_tasks() -> Result<Vec<Task>> {
    let mut all_tasks = Vec::new();
    let mut next_id = 0;

    // Try to discover Just recipes
    if let Ok(just_tasks) = just::discover_tasks() {
        for mut task in just_tasks {
            task.id = next_id;
            next_id += 1;
            all_tasks.push(task);
        }
    }

    // Try to discover Make targets
    if let Ok(make_tasks) = make::discover_tasks() {
        for mut task in make_tasks {
            task.id = next_id;
            next_id += 1;
            all_tasks.push(task);
        }
    }

    if all_tasks.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No tasks discovered. Please ensure you have either a justfile or Makefile in this directory."
        ));
    }

    Ok(all_tasks)
}
