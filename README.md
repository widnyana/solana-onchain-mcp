# solana-onchain-mcp

[![Crates.io](https://img.shields.io/crates/v/solana-onchain-mcp)](https://crates.io/crates/solana-onchain-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Changelog](https://img.shields.io/badge/changelog-v0.2.0-blue)](CHANGELOG.md)

MCP server for Solana blockchain operations.

## Install

### Method 1: Prebuilt Binary (Recommended)

**Warning:** make sure you understand what the bash script doing before executing it

```bash
curl -fsSL https://raw.githubusercontent.com/widnyana/solana-onchain-mcp/refs/heads/main/install.sh | bash
```

This downloads the latest release for your platform and installs to `~/.local/bin`.

### Method 2: Install from crates.io

```bash
cargo install solana-onchain-mcp
```

### Method 3: Build from Source

```bash
cargo install --git https://github.com/widnyana/solana-onchain-mcp
```

Requires Rust 1.85+.

## Setup

Start the server first:

```bash
SOLANA_NETWORK=devnet solana-onchain-mcp --port 3000
```

### Claude Code

`.mcp.json`:

```json
{
  "mcpServers": {
    "solana": {
      "url": "http://localhost:3000/sse"
    }
  }
}
```

### Cursor

`.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "solana": {
      "url": "http://localhost:3000/sse"
    }
  }
}
```

### VS Code

`.vscode/mcp.json` (see [MCP Server docs](https://code.visualstudio.com/docs/copilot/customization/mcp-servers)):

```json
{
  "servers": {
    "solana": {
      "type": "sse",
      "url": "http://localhost:3000/sse"
    }
  }
}
```

### With Keypair (for transfers)

Start the server with keypair support (localhost only):

```bash
SOLANA_NETWORK=devnet SOLANA_KEYPAIR_PATH=/path/to/keypair.json \
  solana-onchain-mcp --http-allow-keypair --accept-risk --host 127.0.0.1 --port 3000
```

Then connect your client to `http://localhost:3000/sse`.

## Tools (19)

### Read (no keypair required)

| Tool | Description |
|------|-------------|
| `get_balance` | SOL balance for address |
| `get_account_info` | Account data and metadata |
| `get_multiple_accounts` | Batch fetch up to 100 accounts |
| `get_token_accounts_by_owner` | SPL token accounts for wallet |
| `get_program_accounts` | Accounts owned by program |
| `get_transaction` | Transaction by signature |
| `get_signatures_for_address` | Transaction history for address |
| `get_signature_status` | Transaction confirmation status |
| `get_slot` | Current slot/block height |
| `simulate_transaction` | Test transaction without signing |
| `inspect_transaction_raw` | Raw transaction with program names |
| `inspect_transaction_humanized` | Human-readable transaction summary |
| `get_server_info` | Server configuration and network info |

### Write (requires keypair)

| Tool | Description |
|------|-------------|
| `transfer_sol` | Transfer SOL |
| `transfer_token` | Transfer SPL tokens |
| `create_associated_token_account` | Create token account |
| `approve_token` | Approve token delegate |
| `revoke_token` | Revoke token delegate |
| `close_token_account` | Close unused token account |

### Mainnet Usage

⚠️ **WARNING:** Mainnet operations involve real assets. Always:
1. Test thoroughly on devnet first
2. Use a dedicated wallet with minimal funds
3. Verify all transaction parameters
4. Set `SOLANA_ACCEPT_RISK=true` or use `--accept-risk` flag

See [Mainnet Usage](USAGE.md#mainnet-usage) for details.

## Docs

See [USAGE.md](USAGE.md) for configuration, parameters, and security.

## Requirements

- Rust 1.85+ (edition 2024)
- Solana RPC access

## License

MIT
