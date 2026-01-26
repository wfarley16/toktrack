.PHONY: check fmt clippy test build release setup

# Run all checks (used by pre-commit)
check: fmt-check clippy test

# Format code
fmt:
	cargo fmt --all

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Run clippy
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	cargo test

# Build debug
build:
	cargo build

# Build release
release:
	cargo build --release

# Setup git hooks
setup:
	git config core.hooksPath .githooks
	@echo "Git hooks configured!"
