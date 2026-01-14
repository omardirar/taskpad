# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Taskpad is a keyboard-driven TUI (Terminal User Interface) task launcher that discovers and runs tasks from multiple task runners. It provides a clean two-pane interface for viewing tasks and streaming their output in real-time. Currently supports Just recipes and Make targets.

## Development Commands

### Building and Running
```bash
# Build for development
cargo build

# Build for release
cargo build --release

# Run taskpad (in a directory with justfile or Makefile)
cargo run

# Install locally
cargo install --path .
```

### Testing and Quality
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check code without building
cargo check

# Run linter
cargo clippy

# Run linter with warnings as errors
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Using Make/Just
The project includes both a Makefile and justfile with convenience targets:
```bash
# Via Make
make build
make test
make fmt
make lint

# Via Just (if installed)
just build
just test
```

## Architecture

### Module Structure

The codebase follows a clean separation of concerns:

- **main.rs**: Entry point, terminal setup, event loop, and event handling (keyboard + mouse)
- **app.rs**: Core application state (`AppState`) and data structures (`Task`, `TaskStatus`, `RunningTask`, `HistoryEntry`)
- **ui.rs**: All TUI rendering logic using Ratatui (pure function of AppState)
- **process.rs**: Task execution as subprocesses with output streaming via channels
- **tasks/**: Task discovery modules
  - **mod.rs**: Unified task discovery interface
  - **just.rs**: Just recipe discovery
  - **make.rs**: Make target discovery

### Key Design Patterns

**State Management**: The entire UI is a pure function of `AppState`. All application state lives in this single struct, making the UI deterministic and easy to reason about.

**Task Execution Model**:
- Tasks run as child processes in separate threads
- stdout/stderr are streamed back to main thread via channels
- Only one task runs at a time (enforced in UI)
- Task logs are persisted per-task-id in `task_logs: HashMap<usize, Vec<String>>`

**Task Identity**: Each task has a stable `id` field assigned during discovery. This ID is used as the key for:
- Per-task log history (`task_logs`)
- Per-task text selections (`task_selections`)
- This allows switching between tasks while preserving their logs and selections

**Multi-Source Task Discovery**:
- `tasks::discover_all_tasks()` combines tasks from all available sources
- Each task carries its `TaskRunner` enum (Just or Make)
- Tasks are displayed with prefixes like `[just]` or `[make]`
- If no tasks are found from any source, returns error

**Event Loop Architecture**:
- Main loop polls for events at ~60 FPS (16ms timeout) for smooth output streaming
- Checks channels for log output and task status updates each frame
- Handles both keyboard events and mouse events (including drag selection)
- Scroll offsets adjust automatically to keep selected task visible

**Text Selection System**:
- Mouse drag in log pane creates text selections
- Selections are stored per-task in `task_selections` HashMap
- Auto-scroll during drag near edges (within 3 rows of top/bottom)
- Copy selected text with `y` (yank) or `Ctrl+C`
- Clear selection with `Esc`

**Scrolling Behavior**:
- Task list: scroll offset auto-adjusts to keep selection visible
- Logs: auto-scroll to bottom when tasks are running, manual scroll disables auto-scroll
- Info box: independent scroll offset for task descriptions
- History: independent scroll offset for task execution history

### Coordinate Systems

**Screen to Log Position Mapping**: The `screen_to_log_position()` function in main.rs converts mouse screen coordinates to log line/column positions. It accounts for:
- Task list width (35 chars)
- Top bar (1 line)
- Info box height (6 lines) if visible
- Border widths
- Current log scroll offset

**Layout Constants**: `TASK_LIST_WIDTH = 35` is defined in both main.rs and ui.rs. Keep these in sync if changing the layout.

## Important Implementation Details

### Adding New Task Runners

To add support for a new task runner:

1. Create `src/tasks/<runner>.rs` with a `discover_tasks()` function
2. Add the runner variant to `TaskRunner` enum in app.rs
3. Implement `prefix()` and `command()` methods for the new variant
4. Update `tasks/mod.rs` to call the new discovery function
5. Update `process.rs` if special handling is needed for execution

### Mouse Event Handling

Mouse events are region-aware:
- Left side splits between task list and history (if visible)
- Right side splits between info box (if visible) and logs
- `get_left_region()` and `get_right_region()` calculate which area was clicked
- Scroll wheel events behave differently based on region

### Clipboard Integration

Uses `arboard` crate for clipboard access:
- Linux requires special handling with `.wait()` to persist clipboard after app exits
- Other platforms use standard `set_text()`

## Dependencies

Key dependencies:
- **ratatui**: TUI framework for rendering
- **crossterm**: Terminal manipulation and event handling
- **tokio**: Async runtime (though most code is synchronous)
- **color-eyre**: Error handling and panic messages
- **arboard**: Cross-platform clipboard access

## Testing Notes

- Tests are in `app.rs` covering basic state management
- No UI tests (Ratatui makes this challenging)
- Process execution is not mocked in tests
- Most testing should be done manually with actual justfiles/Makefiles

## Release Process

Taskpad uses [cargo-dist](https://github.com/axodotdev/cargo-dist) for automated releases and distribution across multiple platforms.

### Creating a Release

1. **Update version** in `Cargo.toml`
   ```bash
   # Update the version field
   version = "0.2.0"  # Use semantic versioning
   ```

2. **Commit changes**
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

3. **Create and push a git tag**
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

4. **Automated workflow**
   - GitHub Actions automatically triggers the release workflow
   - Builds binaries for all target platforms (macOS, Linux, Windows)
   - Creates installers (shell, PowerShell, npm, Homebrew, MSI)
   - Publishes GitHub Release with artifacts
   - Updates Homebrew formula automatically to `omardirar/homebrew-tap`
   - Generates changelog from PR labels and updates CHANGELOG.md

### Distribution Channels

After release, users can install taskpad via:

**Homebrew (macOS/Linux)**:
```bash
brew install omardirar/tap/taskpad
```

Or tap once, then install:
```bash
brew tap omardirar/tap
brew install taskpad
```

**Shell installer (macOS/Linux)**:
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/omardirar/taskpad/releases/latest/download/taskpad-installer.sh | sh
```

**PowerShell (Windows)**:
```powershell
irm https://github.com/omardirar/taskpad/releases/latest/download/taskpad-installer.ps1 | iex
```

**npm**:
```bash
npx taskpad
```

**Direct download**: Download pre-built binaries from [GitHub Releases](https://github.com/omardirar/taskpad/releases)

### Homebrew Tap Setup

Homebrew tap is configured at `omardirar/homebrew-tap`:

1. **Repository**: Already created at `https://github.com/omardirar/homebrew-tap`

2. **Configuration**: cargo-dist is configured in `dist-workspace.toml`:
   - Homebrew installer enabled
   - Tap set to `omardirar/homebrew-tap`
   - Formula will be auto-generated on each release

3. **Automatic Publishing** (optional, for push access):
   - Generate a personal access token with `repo` scope
   - Add as `HOMEBREW_TAP_TOKEN` secret in taskpad repository settings
   - cargo-dist will automatically push formula updates to the tap

The Homebrew formula will be automatically updated by the release workflow.

### Changelog Management

The release workflow automatically updates CHANGELOG.md based on merged pull requests:

**PR Labels**: Add these labels to PRs for automatic categorization:
- `feature` - New features
- `enhancement` - Improvements to existing features
- `bug` - Bug fixes
- `maintenance` - Maintenance tasks
- `docs` - Documentation changes
- `i18n` - Internationalization
- `performance` - Performance improvements
- `ignore-for-release` - Exclude from changelog

**Process**:
1. Merge PRs to main with appropriate labels
2. Create a release tag (e.g., `v0.2.0`)
3. Workflow automatically generates changelog from PRs since last release
4. CHANGELOG.md is updated and committed back to main
5. GitHub Release includes the same changelog content

**Configuration**: Changelog generation is configured in `.github/changelog-config.json`

### Verifying a Release

Before tagging, verify the release plan:
```bash
dist plan
```

Test the release workflow without publishing:
```bash
dist build
```

### cargo-dist Configuration

The distribution configuration is in `dist-workspace.toml`:
- **Installers**: shell, powershell, npm, homebrew, msi
- **Targets**: aarch64/x86_64 for macOS, Linux (gnu/musl), Windows
- **CI**: GitHub Actions workflow at `.github/workflows/release.yml`

To update dist configuration:
```bash
dist init  # Interactive configuration update
```

## Git Workflow

- Main branch: `main`
- Current feature branch: `feat/mouse`
- Recent work includes mouse support, scrolling, and history features
- Do not add "Co-Authored-By" lines to commit messages
