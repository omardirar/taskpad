# Test Directories

This directory contains test environments for each task runner supported by taskpad.

## Directory Structure

- **test-just/**: Contains a justfile for testing Just recipe discovery
- **test-npm/**: Contains a package.json with npm scripts (includes package-lock.json for npm detection)
- **test-cargo/**: Contains a Cargo.toml for testing standard cargo commands
- **test-invoke/**: Contains tasks.py for testing Python Invoke task discovery
- **test-poe/**: Contains pyproject.toml for testing Poe the Poet task discovery
- **test-rake/**: Contains a Rakefile for testing Ruby Rake task discovery
- **test-mixed/**: Contains multiple task files (justfile, Makefile, package.json) to test multi-runner discovery

## Testing

To test each runner:

```bash
cd tests/test-<runner>
cargo run --manifest-path ../../Cargo.toml
```

Or from the project root:
```bash
# Install taskpad locally
cargo install --path .

# Test in each directory
cd tests/test-npm
taskpad

# Test mixed environment
cd tests/test-mixed
taskpad
```

## Notes

- Each test directory is self-contained and demonstrates a specific task runner
- The test-mixed directory shows how taskpad handles multiple task runners in the same project
- All task commands are simple echo statements to verify discovery and execution work correctly
