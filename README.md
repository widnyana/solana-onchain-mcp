# solana-onchain-mcp

[![Crates.io](https://img.shields.io/crates/v/solana-onchain-mcp)](https://crates.io/crates/solana-onchain-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP server for Solana blockchain operations.

## Install

```bash
cargo install --git https://github.com/widnyana/solana-onchain-mcp
```

## Setup

### Claude Code

`.mcp.json`:

```json
{
  "mcpServers": {
    "solana": {
      "command": "solana-onchain-mcp",
      "env": { "SOLANA_NETWORK": "devnet" }
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
      "command": "solana-onchain-mcp",
      "env": { "SOLANA_NETWORK": "devnet" }
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
      "type": "stdio",
      "command": "solana-onchain-mcp",
      "env": { "SOLANA_NETWORK": "devnet" }
    }
  }
}
```

### With Keypair (for transfers)

```json
{
  "mcpServers": {
    "solana": {
      "command": "solana-onchain-mcp",
      "env": {
        "SOLANA_NETWORK": "devnet",
        "SOLANA_KEYPAIR_PATH": "/path/to/keypair.json"
      }
    }
  }
}
```

## Tools (15)

### Read (no keypair)

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

### Write (requires keypair)

| Tool | Description |
|------|-------------|
| `transfer_sol` | Transfer SOL |
| `transfer_token` | Transfer SPL tokens |
| `create_associated_token_account` | Create token account |

## Docs

See [USAGE.md](USAGE.md) for configuration, parameters, and security.

## Requirements

- Rust 1.85+ (edition 2024)
- Solana RPC access

## License

MIT
