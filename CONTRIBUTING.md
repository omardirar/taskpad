# Contributing to Taskpad

Thank you for your interest in contributing to Taskpad! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- **Rust**: 1.70.0 or later
- **just**: For running project tasks (optional but recommended)
  - Install: `cargo install just` or see [just installation docs](https://github.com/casey/just#installation)

### Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/omardirar/taskpad.git
   cd taskpad
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run the application**
   ```bash
   cargo run
   ```

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
│       ├── just.rs      # Just recipe discovery
│       └── make.rs      # Make target discovery
├── Cargo.toml
└── README.md
```

## Architecture

For detailed architecture documentation, see [CLAUDE.md](CLAUDE.md) which includes:
- Module structure and design patterns
- State management approach
- Task execution model
- Event loop architecture
- Mouse and keyboard handling
- Scrolling behavior

## Making Changes

### Code Style

- Follow the existing code style
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality

### Testing

- Write tests for new features
- Ensure all tests pass: `cargo test`
- Manual testing with actual justfiles/Makefiles is recommended

### Commit Messages

Follow conventional commit format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `refactor:` - Code refactoring
- `test:` - Test changes

Example:
```
feat: add support for Make targets
fix: resolve panic when justfile is empty
docs: update installation instructions
```

## Contribution Workflow

### For External Contributors

If you're not a maintainer of this repository:

1. **Fork the repository** to your own GitHub account (required - you don't have direct write access)
2. **Clone your fork** locally
   ```bash
   git clone https://github.com/YOUR-USERNAME/taskpad.git
   cd taskpad
   ```
3. **Create a feature branch** from `main`
   ```bash
   git checkout -b feat/your-feature-name
   ```
4. **Make your changes** following the guidelines above
5. **Push to your fork**
   ```bash
   git push origin feat/your-feature-name
   ```
6. **Open a pull request** from your fork to the main repository
7. **Wait for maintainer review** - only maintainers can approve and merge PRs

### For Maintainers

If you have maintainer/admin access:

- You have direct push access to the repository
- You can approve and merge pull requests
- Follow branch protection rules (create PRs for major changes)
- Ensure CI passes before merging

## Pull Request Process

1. **Fork the repository** (if external contributor) and create a new branch from `main`
2. **Make your changes** following the guidelines above
3. **Test your changes** thoroughly
4. **Add appropriate labels** to your PR:
   - `feature` - New features
   - `enhancement` - Improvements to existing features
   - `bug` - Bug fixes
   - `maintenance` - Maintenance tasks
   - `docs` - Documentation changes
   - `performance` - Performance improvements
5. **Submit your PR** with a clear description of the changes
6. **Wait for approval** - maintainers will review and approve your PR

### PR Labels

Labels are important as they're used for automatic changelog generation:
- `feature` → Features ✨
- `enhancement` → Enhancements 🔥
- `bug` → Fixes 🔧
- `maintenance` → Maintenance ⚙️
- `docs` → Docs 📖
- `i18n` → I18n 🌎
- `performance` → Performance Improvements 📊
- `ignore-for-release` → Excluded from changelog

## Release Process

Releases are managed by maintainers. See [CLAUDE.md](CLAUDE.md#release-process) for details on the automated release workflow.

## Questions or Issues?

- **Bug reports**: Open an issue with detailed reproduction steps
- **Feature requests**: Open an issue describing the feature and use case
- **Questions**: Open a discussion or issue

## Code of Conduct

Be respectful and constructive in all interactions. We're here to build something useful together.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
