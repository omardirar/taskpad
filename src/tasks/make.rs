/// Make target discovery module.
///
/// This module provides functionality to discover available Make targets
/// in the current directory by running `make -qp` and parsing its output.
use crate::app::{Task, TaskRunner};
use color_eyre::eyre::{Result, eyre};
use std::process::Command;

/// Discovers available Make targets in the current directory.
///
/// This function:
/// 1. Checks if `make` is available on PATH
/// 2. Runs `make -qp` to get all targets from the Makefile database
/// 3. Parses the output to extract target names
///
/// # Returns
///
/// Returns `Ok(Vec<Task>)` with discovered tasks, or an error if:
/// - `make` is not installed or not on PATH
/// - The Makefile doesn't exist or is invalid
/// - `make -qp` returns an error
///
/// # Errors
///
/// Returns descriptive errors that can be displayed to the user in the TUI.
pub fn discover_tasks() -> Result<Vec<Task>> {
    // First check if make is available
    let make_check = Command::new("make").arg("--version").output();

    match make_check {
        Err(_) => {
            return Err(eyre!(
                "make not found on PATH. Please install make and try again."
            ));
        }
        Ok(output) if !output.status.success() => {
            return Err(eyre!(
                "make command failed. Please check your make installation."
            ));
        }
        _ => {}
    }

    // Run make -qp to get the makefile database
    // -q: question mode (don't run commands)
    // -p: print database
    // We redirect stderr to suppress "No rule to make target" messages
    let output = Command::new("make")
        .arg("-qp")
        .output()
        .map_err(|e| eyre!("Failed to execute make -qp: {}", e))?;

    // make -qp can return non-zero exit code if no Makefile exists
    // Check stderr for common error messages
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("No such file or directory") && stderr.contains("Makefile") {
        return Err(eyre!("No Makefile found in this directory."));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_make_database(&stdout)
}

/// Parses the output of `make -qp` into a list of tasks.
///
/// The format of `make -qp` includes lines like:
/// ```text
/// target: dependencies
/// .PHONY: clean
/// ```
///
/// This function handles:
/// - Extracting target names from lines with colons
/// - Filtering out special targets (starting with .)
/// - Filtering out implicit rules (containing %)
/// - Filtering out variable assignments
/// - Removing common automatic targets (all, clean, install, etc. are kept as valid targets)
///
/// # Arguments
///
/// * `output` - The stdout from `make -qp`
///
/// # Returns
///
/// A vector of discovered tasks with names.
fn parse_make_database(output: &str) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let mut task_id = 0;
    let mut seen_targets = std::collections::HashSet::new();

    for line in output.lines() {
        let trimmed = line.trim_start();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Skip lines that don't contain a colon (not a target definition)
        if !trimmed.contains(':') {
            continue;
        }

        // Skip variable assignments (contain = before :)
        if let Some(colon_pos) = trimmed.find(':')
            && trimmed[..colon_pos].contains('=')
        {
            continue;
        }

        // Extract target name (everything before the first colon)
        let target = if let Some(colon_pos) = trimmed.find(':') {
            trimmed[..colon_pos].trim()
        } else {
            continue;
        };

        // Skip if target is empty
        if target.is_empty() {
            continue;
        }

        // Skip special targets that start with .
        if target.starts_with('.') {
            continue;
        }

        // Skip implicit rules (contain %)
        if target.contains('%') {
            continue;
        }

        // Skip if target contains spaces (likely not a simple target)
        if target.contains(' ') {
            continue;
        }

        // Skip if we've already seen this target
        if !seen_targets.insert(target.to_string()) {
            continue;
        }

        // Skip common make automatic variables and internal targets
        if matches!(target, "Makefile" | "makefile" | "GNUmakefile") {
            continue;
        }

        // Skip Make internal variables (all caps with underscores)
        if target.chars().all(|c| c.is_uppercase() || c == '_') {
            continue;
        }

        // Skip common internal Make targets
        if matches!(
            target,
            "SUFFIXES"
                | "DEFAULT"
                | "PRECIOUS"
                | "INTERMEDIATE"
                | "SECONDARY"
                | "SECONDEXPANSION"
                | "DELETE_ON_ERROR"
                | "IGNORE"
                | "LOW_RESOLUTION_TIME"
                | "SILENT"
                | "EXPORT_ALL_VARIABLES"
                | "NOTPARALLEL"
                | "ONESHELL"
                | "POSIX"
        ) {
            continue;
        }

        tasks.push(Task {
            id: task_id,
            name: target.to_string(),
            description: None,
            runner: TaskRunner::Make,
        });

        task_id += 1;
    }

    if tasks.is_empty() {
        return Err(eyre!(
            "No make targets discovered. Is there a Makefile with targets in this directory?"
        ));
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_make_database_basic() {
        let output = r#"# Make data base, printed on Sun Jan  1 00:00:00 2024

build: src/main.rs
	cargo build

test:
	cargo test

clean:
	rm -rf target

.PHONY: clean test
"#;

        let tasks = parse_make_database(output).unwrap();

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].name, "build");
        assert_eq!(tasks[1].name, "test");
        assert_eq!(tasks[2].name, "clean");
        assert!(tasks.iter().all(|t| t.runner == TaskRunner::Make));
    }

    #[test]
    fn test_parse_make_database_filters_special_targets() {
        let output = r#"
.PHONY: all
.SUFFIXES: .o .c
%.o: %.c
	gcc -c $<

all: build
build:
	echo "building"
"#;

        let tasks = parse_make_database(output).unwrap();

        // Should only include 'all' and 'build', not .PHONY, .SUFFIXES, or %.o
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].name, "all");
        assert_eq!(tasks[1].name, "build");
    }

    #[test]
    fn test_parse_make_database_no_targets() {
        let output = r#"
# Just variables
CC = gcc
CFLAGS = -Wall
"#;

        let result = parse_make_database(output);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_make_database_deduplicates() {
        let output = r#"
build: main.o
	gcc -o prog main.o

build: utils.o
	gcc -o prog utils.o
"#;

        let tasks = parse_make_database(output).unwrap();
        // Should only have one 'build' target
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "build");
    }

    #[test]
    fn test_parse_make_database_skips_makefiles() {
        let output = r#"
Makefile:
	touch Makefile

build:
	echo "building"
"#;

        let tasks = parse_make_database(output).unwrap();
        // Should not include Makefile as a target
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "build");
    }
}
