.PHONY: default install install-separate install-all build clippy fmt fmt-check test check

default:
	@echo "Available targets:"
	@echo "  install            Install the unified nf3 binary"
	@echo "  install-separate   Install legacy binaries"
	@echo "  install-all        Install both unified and legacy binaries"
	@echo "  build              Build the entire workspace in release mode"
	@echo "  clippy             Run Clippy with warnings as errors"
	@echo "  fmt                Format the code"
	@echo "  fmt-check          Check formatting"
	@echo "  test               Run all tests"
	@echo "  check              Run fmt-check, clippy, and tests"

install:
	cargo install --path ./nf3

install-separate:
	cargo install --path ./f3read
	cargo install --path ./f3write
	# cargo install --path ./f3probe
	# cargo install --path ./f3fix
	# cargo install --path ./f3brew

install-all: install install-separate

build:
	cargo build --workspace --release

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

test:
	cargo test --workspace --all-features --no-fail-fast

check: fmt-check clippy test
