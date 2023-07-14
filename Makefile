fmt:
	cargo fmt
test:
	cargo test --features "image"
lint:
	cargo clippy --features "image"
udeps:
	cargo +nightly udeps