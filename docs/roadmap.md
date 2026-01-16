# Taskpad Roadmap

This document outlines the planned features and improvements for Taskpad. The roadmap is subject to change based on community feedback and priorities.

## Current Status

**Version**: 0.1.0
**Stable Features**:
- Just and Make task runner support
- Keyboard-driven interface with Vi-style bindings
- Mouse support (selection, scrolling, text selection)
- Real-time output streaming
- Task execution history
- Info box with task descriptions
- Cross-platform support (macOS, Linux, Windows)

---

## Near Term (v0.2.x - v0.3.x)

### Additional Task Runners
- **npm/pnpm/yarn scripts** (package.json)
- **Cargo tasks** (Cargo.toml with custom commands)
- **Python task runners** (Invoke, Poe the Poet)
- **Rake** (Ruby tasks)

### User Experience Improvements
- **Task filtering/search**: Quick search within task list (`/` key)
- **Task favoriting**: Pin frequently used tasks to the top
- **Keyboard shortcuts customization**: User-defined keybindings
- **Task grouping**: Visual separation of tasks by runner type
- **Syntax highlighting**: Colored output for common patterns (errors, warnings, etc.)

### Configuration
- **Config file support**: TOML/YAML configuration in `~/.config/taskpad/`
- **Task Arguments:** Add dialogue to pass optional arguments/parameters to tasks.
- **Theme support**: Customizable colors and UI elements
- **Default task runner priority**: Choose which runners to show first
- **Output formatting options**: Control log display style

---

## Medium Term (v0.4.x - v0.6.x)

### Parallel Execution
- **Multiple task execution**: Run multiple tasks concurrently
- **Task queue**: Queue tasks to run in sequence
- **Task dependencies**: Automatically run prerequisite tasks
- **Resource limits**: Configure max concurrent tasks

### Enhanced Task Management
- **Task arguments**: Pass arguments to tasks at runtime
- **Environment variables**: Set/override env vars per task
- **Task templates**: Create and run ad-hoc commands
- **Task history persistence**: Save history across sessions

### Workspace Features
- **Multi-directory support**: Monitor multiple project directories
- **Workspace switcher**: Quick navigation between projects
- **Recent projects**: Jump to recently used taskfiles
- **Monorepo support**: Discover tasks in subdirectories

### Integration Features
- **Git integration**: Show branch/status, pre-commit hooks
- **CI/CD indicators**: Highlight tasks that match CI pipeline
- **Notification support**: Desktop notifications for task completion
- **Export logs**: Save task output to files

---

## Long Term (v0.7.x - v1.0+)

### Advanced Features
- **Task scheduling**: Cron-like scheduled task execution
- **Watch mode**: Re-run tasks on file changes
- **Task templates library**: Share and import task collections
- **Remote task execution**: Run tasks on remote servers via SSH
- **Container integration**: Discover and run Docker Compose services

### Performance & Scalability
- **Lazy loading**: Handle projects with 100+ tasks efficiently
- **Output buffering optimization**: Improve streaming for high-volume output
- **Memory usage optimization**: Reduce footprint for long-running sessions
- **Incremental task discovery**: Watch for taskfile changes

### Developer Experience
- **Plugin system**: Extensible architecture for custom task runners
- **API/IPC**: Control Taskpad from other tools
- **Scripting support**: Automate Taskpad actions
- **Test coverage**: Comprehensive test suite

### UI/UX Polish
- **Split panes**: Multiple log views side-by-side
- **Tabs**: Multiple task outputs in tabs
- **Custom layouts**: User-defined pane arrangements
- **Accessibility**: Screen reader support, high contrast themes
- **i18n**: Internationalization support

---

## Future Considerations

These ideas are being considered but not yet committed to a timeline:

- **LSP integration**: Task validation and autocomplete
- **Task analytics**: Track execution times, success rates
- **Team collaboration**: Share task history/favorites
- **Cloud sync**: Sync config across machines
- **Mobile companion**: View task status on mobile devices
- **VS Code extension**: Taskpad integration in VS Code
- **Shell completions**: Tab completion for taskpad CLI args

---

## Community Input

We welcome feedback on this roadmap. Please:

- **Vote on features**: Use GitHub reactions on issues to show interest
- **Suggest features**: Open an issue with the `enhancement` label
- **Contribute**: Check [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines

---

## Release Philosophy

- **Semantic versioning**: Major.Minor.Patch (e.g., 1.0.0)
- **Stability focus**: Features are thoroughly tested before release
- **Backward compatibility**: Breaking changes only in major versions
- **Regular releases**: Aim for minor releases every 1-2 months

---

## How to Track Progress

- **GitHub Issues**: Each feature will have a tracking issue
- **Milestones**: Releases are organized in GitHub Milestones
- **Project Board**: Check the [GitHub Project](https://github.com/omardirar/taskpad/projects) for status
- **Changelog**: See [CHANGELOG.md](../CHANGELOG.md) for completed features

---

*Last updated: 2026-01-16*
