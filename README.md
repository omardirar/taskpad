# Taskpad

[![CI](https://github.com/omzification/taskpad/workflows/CI/badge.svg)](https://github.com/omzification/taskpad/actions)

A keyboard-driven TUI task launcher with first-class support for [just](https://github.com/casey/just) recipes.

## What is Taskpad?

Taskpad is a terminal user interface (TUI) application that helps you discover and run tasks in your project. Version 0 focuses on providing a smooth, keyboard-driven experience for working with `just` recipes.

### Features

- 🚀 **Fast & Responsive**: Keyboard-driven interface with no mouse support needed
- 📋 **Just Integration**: Automatically discovers and lists all `just` recipes in your project
- 📊 **Real-time Output**: Stream task output (stdout/stderr) in real-time
- 🎯 **Simple UI**: Clean two-pane layout showing tasks and logs
- ⌨️ **Intuitive Controls**: Vi-style keybindings and arrow keys supported

## Requirements

- **Rust**: For building and installing Taskpad (1.70.0 or later recommended)
- **just**: Must be installed and available on your PATH
  - Install just: `cargo install just` or see [just installation docs](https://github.com/casey/just#installation)

## Installation

### From Source

```bash
git clone https://github.com/omzification/taskpad.git
cd taskpad
cargo install --path .
```

Or build for development:

```bash
cargo build --release
# Binary will be at ./target/release/taskpad
```

## Usage

Navigate to a directory containing a `justfile` and run:

```bash
taskpad
```

Taskpad will:
1. Discover all available `just` recipes
2. Display them in a list on the left pane
3. Show task output in the right pane when you run a task

### Keyboard Controls

| Key(s) | Action |
|--------|--------|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Run the selected task |
| `r` | Reload tasks from justfile |
| `c` | Clear the log pane |
| `q` | Quit Taskpad |

### Notes

- **One task at a time**: For v0, Taskpad only runs one task at a time. If a task is running, you'll need to wait for it to complete before starting another.
- **Exit codes**: Task success/failure status is shown with exit codes in the status bar.
- **Stderr highlighting**: Lines from stderr are prefixed with `[stderr]` and highlighted in red.

## Example

Create a simple `justfile`:

```justfile
# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean
```

Then run `taskpad` in the same directory. You'll see all three recipes listed, and you can select and run them with the keyboard.

## Error Handling

If Taskpad encounters an error, it will display a helpful message:

- **`just` not found**: Install `just` and ensure it's on your PATH
- **No justfile**: Make sure you're in a directory with a `justfile` or `Justfile`
- **No tasks discovered**: Your justfile might be empty or have syntax errors

## Project Structure

```
taskpad/
├── src/
│   ├── main.rs          # Entry point and event loop
│   ├── app.rs           # Application state and data structures
│   ├── ui.rs            # TUI rendering logic
│   ├── process.rs       # Task execution and output streaming
│   └── tasks/
│       ├── mod.rs       # Task discovery module interface
│       └── just.rs      # Just recipe discovery
├── Cargo.toml
└── README.md
```

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo check
cargo clippy
```

### Building for Release

```bash
cargo build --release
```

## Future Enhancements

Version 0 is intentionally minimal. Potential future features include:

- Support for other task runners (npm, make, rake, etc.)
- Running multiple tasks concurrently
- Task history and favorites
- Custom keybindings
- Theme customization
- Search/filter tasks

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Designed for [just](https://github.com/casey/just) task runner
- Inspired by modern TUI tools like lazygit and bottom
