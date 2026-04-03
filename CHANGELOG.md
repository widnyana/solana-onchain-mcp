# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-04-04

### Added
- `query_transactions` tool: filter and classify transactions with cursor-based MCP pagination
- Improved tool descriptions with explicit LLM guidance

### Changed
- Dropped stdio transport; server is now HTTP-only (Streamable HTTP via `/mcp`)
- Removed `--http` flag; HTTP is now the default and only transport
- Tightened tool descriptions for `get_balance`, `get_signatures_for_address`, `create_associated_token_account`
- Client configuration now uses `url: http://localhost:<port>/mcp` instead of stdio `command`

### Breaking
- Stdio transport removed — existing stdio-based client configs must switch to HTTP:
  `{ "url": "http://localhost:3000/mcp" }`

### Fixed
- `query_transactions`: failed_count now counted from Phase 1 scan (not Phase 2 filter)
- `query_transactions`: removed stale `last_cursor` field from response output

## [0.2.0] - 2026-03-25

### Added
- Mainnet usage warnings when operating on mainnet or custom networks
- Explicit warnings at 5 strategic points in the codebase
- `get_server_info` tool for server configuration introspection
- `approve_token` tool for SPL token delegate approvals
- `revoke_token` tool for revoking token delegates
- `close_token_account` tool for closing unused token accounts
- Installation via install.sh script for prebuilt binaries

### Changed
- Updated README to document all 19 available tools (previously listed 15)
- Expanded HTTP mode documentation with use cases and configuration
- Improved mainnet safety warnings in documentation
- Tool count updated throughout documentation (15 -> 19)

### Fixed
- Documentation now correctly lists all available token management tools
- Installation instructions now prioritize install.sh over cargo install

### Security
- Added prominent warnings when using mainnet or custom networks
- HTTP mode keypair protection requires explicit --accept-risk and localhost
- Mainnet operations require both CLI flag and environment variable confirmation

## [0.1.0] - Initial Release

### Added
- Initial release with 15 Solana MCP tools
- Support for mainnet, devnet, testnet, and custom RPC endpoints
- Keypair-based write operations with safety guards
- Transaction inspection tools (raw and humanized)
- HTTP and stdio transport modes
