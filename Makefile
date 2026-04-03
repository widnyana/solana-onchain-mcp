inspect:
	@cargo build
	@cargo run -- --port 3000 & MCP_PID=$$!; \
	trap "kill $$MCP_PID 2>/dev/null" EXIT INT TERM; \
	sleep 1; \
	pnpm dlx @modelcontextprotocol/inspector@latest; \
	kill $$MCP_PID 2>/dev/null

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

clippy:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

dev:
	@echo "Running tryout..."
	@cargo run -- --port 3000 --log-level debug
