/// Task discovery modules.
///
/// This module provides functionality for discovering tasks from various sources.
/// Supports Just recipes, Make targets, npm/pnpm/yarn scripts, Cargo tasks,
/// Python task runners (Invoke, Poe), and Rake tasks.
use crate::app::Task;
use color_eyre::eyre::Result;

pub mod just;
pub mod make;
pub mod npm;

/// Discovers tasks from all available sources.
///
/// This function attempts to discover tasks from:
/// 1. Just recipes (if justfile exists)
/// 2. Make targets (if Makefile exists)
/// 3. npm/pnpm/yarn scripts (if package.json exists)
/// 4. Cargo tasks (if cargo is available)
/// 5. Python Invoke tasks (if invoke is available)
/// 6. Python Poe tasks (if poe is available)
/// 7. Rake tasks (if rake is available)
///
/// Tasks from all sources are combined into a single list with unique IDs.
/// Tasks are prefixed in the UI based on their TaskRunner type.
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

    // Try to discover npm/pnpm/yarn scripts
    if let Ok(npm_tasks) = npm::discover_tasks() {
        for mut task in npm_tasks {
            task.id = next_id;
            next_id += 1;
            all_tasks.push(task);
        }
    }

    if all_tasks.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No tasks discovered. Please ensure you have a task file (justfile, Makefile, package.json, etc.) in this directory."
        ));
    }

    Ok(all_tasks)
}
