# Run unit tests only (no Docker needed)
test-unit:
    cargo test --bins

# Start test database, run integration tests, stop database
test-integration:
    docker compose -f docker-compose.test.yml up -d --wait
    (cargo test --test postgres_seed -- --ignored; status=$?; docker compose -f docker-compose.test.yml down; exit $status)

# Run all tests
test-all: test-unit test-integration

# Quick compile check
check:
    cargo check

# Run the app in debug mode
run:
    cargo run

# Build release
release:
    cargo build --release

# Format code
fmt:
    cargo fmt

# Lint
lint:
    cargo clippy --all-targets -- -D warnings
