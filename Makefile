fmt:
	cargo fmt
test:
	cargo test --features "image"
lint:
	cargo clippy
udeps:
	cargo +nightly udeps
bench:
	cargo bench