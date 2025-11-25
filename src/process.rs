/// Process execution module.
///
/// This module handles running tasks as subprocesses and streaming
/// their output back to the main thread via channels.

use crate::app::{Task, TaskStatus};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

/// Spawns a task as a subprocess and streams its output.
///
/// This function:
/// 1. Spawns `just <recipe-name>` as a child process
/// 2. Captures both stdout and stderr
/// 3. Streams output line-by-line to `log_tx`
/// 4. Sends final status to `status_tx` when the process exits
///
/// The function spawns a separate thread to handle I/O, so it returns immediately
/// and doesn't block the caller.
///
/// # Arguments
///
/// * `task` - The task to run
/// * `log_tx` - Channel sender for log lines
/// * `status_tx` - Channel sender for final status updates
///
/// # Panics
///
/// This function may panic if the channels are disconnected, which would indicate
/// a programming error (the main thread dropped its receivers).
pub fn run_task(
    task: Task,
    log_tx: Sender<String>,
    status_tx: Sender<TaskStatus>,
) {
    thread::spawn(move || {
        // Send initial log message
        let _ = log_tx.send(format!("Starting task: {}", task.name));

        // Spawn the just command
        let mut child = match Command::new("just")
            .arg(&task.name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                let _ = log_tx.send(format!("ERROR: Failed to spawn process: {}", e));
                let _ = status_tx.send(TaskStatus::Failed(-1));
                return;
            }
        };

        // Get stdout and stderr handles
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Create channels for merging stdout and stderr
        let (merged_tx, merged_rx) = std::sync::mpsc::channel();

        // Spawn thread for stdout
        let stdout_tx = merged_tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        let _ = stdout_tx.send(line);
                    }
                    Err(e) => {
                        let _ = stdout_tx.send(format!("ERROR reading stdout: {}", e));
                        break;
                    }
                }
            }
        });

        // Spawn thread for stderr
        let stderr_tx = merged_tx;
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        // Prefix stderr lines to distinguish them
                        let _ = stderr_tx.send(format!("[stderr] {}", line));
                    }
                    Err(e) => {
                        let _ = stderr_tx.send(format!("ERROR reading stderr: {}", e));
                        break;
                    }
                }
            }
        });

        // Forward merged output to log channel
        // We need to stop when both stdout and stderr threads have finished
        // For simplicity, we'll read until the child process exits
        let log_tx_clone = log_tx.clone();
        let _reader_thread = thread::spawn(move || {
            while let Ok(line) = merged_rx.recv() {
                let _ = log_tx_clone.send(line);
            }
        });

        // Wait for the child process to exit
        match child.wait() {
            Ok(status) => {
                // Give a moment for remaining output to be processed
                thread::sleep(std::time::Duration::from_millis(100));

                let exit_code = status.code().unwrap_or(-1);
                let _ = log_tx.send(format!("Task exited with code: {}", exit_code));

                let task_status = if status.success() {
                    TaskStatus::Success(exit_code)
                } else {
                    TaskStatus::Failed(exit_code)
                };

                let _ = status_tx.send(task_status);
            }
            Err(e) => {
                let _ = log_tx.send(format!("ERROR: Failed to wait for process: {}", e));
                let _ = status_tx.send(TaskStatus::Failed(-1));
            }
        }

        // The reader thread will exit when merged_tx is dropped (when stdout/stderr threads finish)
    });
}
