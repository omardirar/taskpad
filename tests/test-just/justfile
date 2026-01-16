# Taskpad test justfile
# This file contains sample recipes for testing Taskpad

# Build the project
build:
    cargo build

# Run tests
test:
    cargo test

# Check code quality
check:
    cargo check
    cargo clippy

# Run the application
run:
    cargo run

# Clean build artifacts
clean:
    cargo clean

# Show a simple hello message
hello:
    echo "Hello from Taskpad!"
    echo "This is a test recipe"

# A task that produces a lot of output
verbose:
    @echo "Generating verbose output..."
    @for i in {1..50}; do echo "Line $i: This is a test of streaming output"; done
    @echo "Done!"

# A task that fails
fail:
    echo "This task will fail"
    exit 1

# A long-running task
long:
    echo "Starting long-running task..."
    sleep 3
    echo "Still running..."
    sleep 2
    echo "Almost done..."
    sleep 1
    echo "Complete!"
