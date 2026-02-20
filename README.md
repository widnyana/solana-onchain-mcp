# solana-onchain-mcp

[![Crates.io](https://img.shields.io/crates/v/solana-onchain-mcp)](https://crates.io/crates/solana-onchain-mcp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

MCP server for Solana blockchain operations with AI assistants.

## Features

- Query SOL balances and transaction details
- Get current slot/block height
- Transfer SOL and SPL tokens
- Multi-network support (mainnet, devnet, testnet, custom)
- Security-hardened with mainnet guard

## Installation

```bash
cargo install --git https://github.com/widnyana/solana-onchain-mcp
```

## Quick Start

### Claude Code

Create `.mcp.json` in your project:

```json
{
  "mcpServers": {
    "solana": {
      "command": "solana-onchain-mcp",
      "env": {
        "SOLANA_NETWORK": "devnet"
      }
    }
  }
}
```

### Cursor

Create `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "solana": {
      "command": "solana-onchain-mcp",
      "env": {
        "SOLANA_NETWORK": "devnet"
      }
    }
  }
}
```

### VS Code

Add to `settings.json` (requires [MCP extension](https://marketplace.visualstudio.com/items?itemName=anthropic.mcp)):

```json
{
  "mcp.servers": {
    "solana": {
      "command": "solana-onchain-mcp",
      "env": {
        "SOLANA_NETWORK": "devnet"
      }
    }
  }
}
```

### With Keypair (for transfers)

To enable transfer operations, set the keypair path:

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

## Documentation

See [USAGE.md](USAGE.md) for:
- Configuration options (environment variables, CLI flags)
- Tool reference (parameters, types)
- Commitment levels
- Security guide

## Requirements

- Rust 1.85+ (edition 2024)
- Solana RPC access (public or custom endpoint)

## License

MIT
