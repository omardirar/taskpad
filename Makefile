# Test Makefile for Taskpad
# This file contains sample targets for testing Make integration

.PHONY: all build test clean install help

all: build test

build:
	@echo "Building the project..."
	cargo build --release

test:
	@echo "Running tests..."
	cargo test

clean:
	@echo "Cleaning up..."
	cargo clean

install:
	@echo "Installing..."
	cargo install --path .

help:
	@echo "Available targets:"
	@echo "  all     - Build and test"
	@echo "  build   - Build the project"
	@echo "  test    - Run tests"
	@echo "  clean   - Clean build artifacts"
	@echo "  install - Install the binary"
	@echo "  help    - Show this help message"

run:
	@echo "Running taskpad..."
	cargo run

fmt:
	@echo "Formatting code..."
	cargo fmt

lint:
	@echo "Running linter..."
	cargo clippy -- -D warnings
