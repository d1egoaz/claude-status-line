# Default recipe
default:
    @just --list

# Build release binary
build:
    cargo build --release

# Build debug binary (faster compile, slower runtime)
debug:
    cargo build

# Run tests
test:
    cargo test

# Test binary with sample JSON
try:
    @echo '{"model":{"display_name":"Opus 4.5"},"cost":{"total_cost_usd":1.67},"cwd":"/foo/bar/project","context_window":{"context_window_size":200000,"used_percentage":16}}' | ./target/release/statusline

# Test with empty JSON (fallback values)
try-empty:
    @echo '{}' | ./target/release/statusline

# Show binary size
size:
    @ls -lh ./target/release/statusline | awk '{print $5}'

# Clean build artifacts
clean:
    cargo clean

# Rebuild from scratch
rebuild: clean build

# Check for compilation errors without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Lint code
lint:
    cargo clippy -- -W clippy::pedantic
