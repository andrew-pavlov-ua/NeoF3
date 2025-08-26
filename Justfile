install: 
	cargo build --release
	cargo install --path=./f3read
	cargo install --path=./f3write

read_test:
	cargo test -p f3read -- --nocapture
