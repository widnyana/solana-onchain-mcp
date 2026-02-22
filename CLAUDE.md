# CLAUDE.md - Project Context for Claude Code

## Project Overview

**solana-onchain-mcp** - A Rust-based MCP (Model Context Protocol) server that provides AI agents with tools to interact with the Solana blockchain.

## Technology Stack

- **Language**: Rust (Edition 2024)
- **MCP SDK**: `rust-mcp-sdk` v0.8.3 with `hyper-server` and `streamable-http` features
- **Solana**: `solana-client` v2.0, `solana-sdk` v2.0, `spl-token` v7.0, `spl-associated-token-account` v6.0
- **Async Runtime**: Tokio

## Project Structure

```
src/
  main.rs          # Entry point, CLI args, HTTP/stdio server modes
  config.rs        # Configuration (RPC URL, network type, keypair path)
  error.rs         # Error types with RPC message sanitization
  handler.rs       # MCP tool handler, dispatches to tool implementations
  lib.rs           # Library exports (ParsePubkeyExt)
  keypair.rs       # Keypair loading from file
  utils.rs         # ParsePubkeyExt trait for DRY pubkey parsing
  rpc/
    mod.rs         # RPC module
    client.rs      # Solana RPC client wrapper
  tools/
    mod.rs         # Tool exports, helpers (json_to_text, text_result, json_result)
    ...
    tx_inspector/
      mod.rs       # Module exports, get_program_name, format_instruction_error
      ...
tests/
  integration.rs
  tx_inspector_tests.rs
```

## Available Tools

### Read Tools (no keypair required)
- `get_balance` - Account balance in SOL
- `get_account_info` - Raw account data with encoding options
- `get_multiple_accounts` - Batch account queries (max 100)
- `get_token_accounts_by_owner` - SPL token accounts for an owner
- `get_program_accounts` - Program account search (filters required)
- `get_transaction` - Transaction details by signature
- `get_signatures_for_address` - Transaction history for an address
- `get_signature_status` - Transaction confirmation status
- `get_slot` - Current slot number
- `simulate_transaction` - Transaction simulation

### Write Tools (require keypair)
- `transfer_sol` - Transfer SOL to recipient
- `transfer_token` - Transfer SPL tokens
- `create_associated_token_account` - Create ATA for token mint

### Inspector Tools
- `inspect_transaction_raw` - Raw transaction analysis
- `inspect_transaction_humanized` - Human-readable transaction breakdown with program identification

## Configuration

### Environment Variables
- `SOLANA_NETWORK` - Network selection: `mainnet`, `devnet`, `testnet`, or custom URL
- `SOLANA_KEYPAIR_PATH` - Path to keypair JSON file
- `SOLANA_ACCEPT_RISK` - Set to `true` to enable write operations on mainnet/custom networks

### CLI Flags
```
--accept-risk          Accept risk of using private key on mainnet/custom networks
--http                 Enable HTTP transport mode (default: stdio)
--port <PORT>          HTTP port (default: 3000), requires --http
--host <HOST>          HTTP host (default: 127.0.0.1), requires --http
--http-allow-keypair   Allow keypair in HTTP mode (requires --http, --accept-risk, and localhost)
```

**Note**: `--http-allow-keypair` has no effect without `--http`. HTTP mode is opt-in via `--http` flag.

### Network Selection
Network is configured via `SOLANA_NETWORK` environment variable:
- `devnet` (default) - Solana devnet
- `mainnet` - Solana mainnet-beta
- `testnet` - Solana testnet
- `http://...` or `https://...` - Custom RPC URL

## Development Commands

```bash
# Build
cargo build

# Run in stdio mode (default, devnet)
cargo run

# Run with mainnet
SOLANA_NETWORK=mainnet cargo run

# Run with custom RPC
SOLANA_NETWORK=http://localhost:8899 cargo run

# Run in HTTP mode
cargo run -- --http --port 3000

# Run in HTTP mode with keypair (localhost only, requires all three flags)
cargo run -- --http --http-allow-keypair --accept-risk --host 127.0.0.1

# Test
cargo test

# Lint
cargo clippy -- -D warnings
```

## Security Notes

- **Keypair disabled by default in HTTP mode** - requires explicit `--http-allow-keypair`
- **`--http-allow-keypair` requires**:
  - `--accept-risk` flag
  - `--host 127.0.0.1` (localhost only)
- **Mainnet/custom network protection** - requires `--accept-risk` or `SOLANA_ACCEPT_RISK=true`
- **RPC error sanitization** - removes sensitive data patterns from error messages
- **`get_program_accounts` requires filters** - prevents memory exhaustion attacks
- **Private IP blocking** - HTTPS URLs to private IPs are rejected

## Key Exports

From `src/lib.rs`:
- `ParsePubkeyExt` trait - extends `str` with `parse_pubkey()` method

From `src/tools/tx_inspector/`:
- `InspectTransactionRawTool`, `InspectTransactionHumanizedTool`
- `ParsedInstructionData`, `ProgramCategory`
- `decode_instruction`, `identify_program`, `interpret_error`
- `get_program_name`, `format_instruction_error`

## Git Workflow

- Feature branches: `feat/*`
- Fix branches: `feat/fix-*` or `fix/*`
- Refactor branches: `refactor/*`
- Merge strategy: Cherry-pick for fixes, merge commits for features

### Worktree Pattern (REQUIRED for feature work)

ALWAYS use git worktrees for feature development. Never work directly on main.

```bash
# Create worktree with new branch
git worktree add .worktree/feat-my-feature -b feat/my-feature

# Work in the worktree
cd .worktree/feat-my-feature
# ... make changes, commit, push ...

# After merge, clean up
git worktree remove .worktree/feat-my-feature
git worktree prune
```

**Rules:**
- All worktrees MUST be inside `.worktree/` directory
- NEVER use `rm -rf` to remove worktrees - use `git worktree remove`
- Always run `git worktree prune` after cleanup
- `.worktree/` should be in `.gitignore`
