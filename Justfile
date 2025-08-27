install: 
	cargo build --release
	cargo install --path=./f3read
	cargo install --path=./f3write

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
	cargo fmt &
	cargo fmt --all -- --check

test:
	cargo test --workspace --all-features --no-fail-fast
