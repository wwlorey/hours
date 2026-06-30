# Default: build and install the hours binary
default: install

# Install the `hours` binary into ~/.cargo/bin
install:
    cargo install --path . --force

# Compile the project
build:
    cargo build

# Run all tests
test:
    cargo test --workspace

# Lint with clippy, treating warnings as errors
lint:
    cargo clippy --workspace -- -D warnings

# Format all code
fmt:
    cargo fmt --all
