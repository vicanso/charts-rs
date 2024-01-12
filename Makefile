fmt:
	cargo fmt
test:
	cargo test --features "image-encoder"
lint:
	cargo clippy
udeps:
	cargo +nightly udeps
bench:
	cargo bench --features "image-encoder"