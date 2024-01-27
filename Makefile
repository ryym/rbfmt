.PHONY: test
test:
	RUST_MIN_STACK=4194304 cargo test
