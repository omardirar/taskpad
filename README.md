# Taskpad

<div align="center">

<img alt="GitHub Actions Workflow Status" src="https://img.shields.io/github/actions/workflow/status/omardirar/taskpad/ci.yml"> <img alt="GitHub Release Date" src="https://img.shields.io/github/release-date/omardirar/taskpad?link=https%3A%2F%2Fgithub.com%2Fomardirar%2Ftaskpad%2Freleases%2Flatest"> <img alt="GitHub Release" src="https://img.shields.io/github/v/release/omardirar/taskpad?logoColor=blue&link=https%3A%2F%2Fgithub.com%2Fomardirar%2Ftaskpad%2Freleases%2Flatest"> <img alt="GitHub Downloads (all assets, all releases)" src="https://img.shields.io/github/downloads/omardirar/taskpad/total">

A keyboard-driven TUI task launcher that discovers and runs tasks from multiple task runners.

<img src="docs/assets/demo.gif" alt="demo" style="max-width: 600px; width: auto;">
</div>

## Table of Contents

- [What is Taskpad?](#what-is-taskpad)
- [Features](#features)
- [Installation](#installation)
- [Requirements](#requirements)
- [Usage](#usage)
- [Example](#example)
- [Error Handling](#error-handling)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

## What is Taskpad?

Taskpad is a terminal user interface (TUI) application that helps you discover and run tasks in your project. It provides a clean, two-pane interface for viewing tasks and streaming their output in real-time.

### Features

- 🚀 **Fast & Responsive**: Keyboard-driven interface with smooth scrolling
- 📋 **Multi-Runner Support**: Works with Just recipes and Make targets
- 📊 **Real-time Output**: Stream task output (stdout/stderr) in real-time
- 🖱️ **Mouse Support**: Click to select tasks, scroll logs, and select text
- 🎯 **Simple UI**: Clean layout with task list, logs, info box, and history
- ⌨️ **Intuitive Controls**: Vi-style keybindings and arrow keys supported
- 📝 **Text Selection**: Mouse-based text selection with copy support

## Installation

### Homebrew (macOS/Linux)

```bash
brew install omardirar/tap/taskpad
```

Or tap once, then install:

```bash
brew tap omardirar/tap
brew install taskpad
```

### Shell Installer (macOS/Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/omardirar/taskpad/releases/latest/download/taskpad-installer.sh | sh
```

### PowerShell (Windows)

```powershell
irm https://github.com/omardirar/taskpad/releases/latest/download/taskpad-installer.ps1 | iex
```

### npm

```bash
npx taskpad
```

### From Source

```bash
git clone https://github.com/omardirar/taskpad.git
cd taskpad
cargo install --path .
```

### Direct Download

Download pre-built binaries from [GitHub Releases](https://github.com/omardirar/taskpad/releases).

## Requirements

- **just** or **make**: At least one task runner must be installed
  - Install just: `cargo install just` or see [just installation docs](https://github.com/casey/just#installation)
  - make is usually pre-installed on most systems

## Usage

Navigate to a directory containing a `justfile` or `Makefile` and run:

```bash
taskpad
```

Taskpad will:
1. Discover all available tasks from Just and/or Make
2. Display them in a list on the left pane
3. Show task output in the right pane when you run a task
4. Keep a history of task executions at the bottom

### Keyboard Controls

| Key(s) | Action |
|--------|--------|
| `↑` / `k` | Move selection up in task list |
| `↓` / `j` | Move selection down in task list |
| `Enter` | Run the selected task |
| `r` | Reload tasks from files |
| `c` | Clear the log pane |
| `i` | Toggle info box (task description) |
| `h` | Toggle history panel |
| `y` or `Ctrl+C` | Copy selected text to clipboard |
| `Esc` | Clear text selection |
| `q` | Quit Taskpad |

### Mouse Controls

- **Click** task in list to select it
- **Scroll wheel** to scroll task list, history, info box, or logs
- **Click and drag** in log pane to select text
- **Double-click** a history entry to re-run that task

### Notes

- **One task at a time**: Taskpad runs one task at a time. Wait for completion before starting another.
- **Exit codes**: Task success/failure status is shown in the status bar.
- **Multi-source**: Tasks from both Just and Make are shown together with `[just]` or `[make]` prefixes.

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

Then run `taskpad` in the same directory. You'll see all three recipes listed, and you can select and run them with the keyboard or mouse.

## Error Handling

If Taskpad encounters an error, it will display a helpful message:

- **No task runner found**: Install `just` or ensure `make` is available
- **No tasks discovered**: Your justfile/Makefile might be empty or have syntax errors

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, coding guidelines, and how to submit pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- Designed for [just](https://github.com/casey/just) and [make](https://www.gnu.org/software/make/) task runners
- Inspired by modern TUI tools like lazygit and bottom
