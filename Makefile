fmt:
	@echo "Formatting Rust code with nightly..."
	@cargo +nightly fmt --all

fmt-check:
	@echo "Checking Rust code formatting with nightly..."
	@cargo +nightly fmt --all -- --check

# Quick format check using rustup
fmt-quick:
	@echo "Quick formatting with nightly..."
	@rustup run nightly cargo fmt --all
