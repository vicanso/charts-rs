fmt:
	cargo fmt
test:
	cargo test --features "image-encoder"
lint:
	cargo clippy --features=image-encoder --all-targets --all -- --deny=warnings
udeps:
	cargo +nightly udeps
bench:
	cargo bench --features "image-encoder"
