# Usage Guide

Complete reference for configuring and using solana-onchain-mcp.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SOLANA_NETWORK` | `devnet` | Network: `mainnet`, `devnet`, `testnet`, or custom RPC URL |
| `SOLANA_KEYPAIR_PATH` | (none) | Path to keypair JSON file (required for transfers) |
| `SOLANA_ACCEPT_RISK` | `false` | Set `true` to enable transfers on mainnet/custom networks |
| `RUST_LOG` | `info` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

### CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--accept-risk` | false | Accept risk of transfers on mainnet/custom networks |
| `--http` | false | Enable HTTP transport (planned) |
| `--port` | 3000 | HTTP server port (planned) |
| `--host` | 127.0.0.1 | HTTP bind address (planned) |
| `--http-allow-keypair` | false | Allow keypair in HTTP mode (planned) |

### Networks

| Network | RPC URL |
|---------|---------|
| `mainnet` | `https://api.mainnet-beta.solana.com` |
| `devnet` | `https://api.devnet.solana.com` |
| `testnet` | `https://api.testnet.solana.com` |
| Custom | Any `http://` or `https://` URL |

**Note:** `https://` URLs cannot point to private IPs (localhost, 10.x, 172.16-31.x, 192.168.x). Use `http://` for local development.

## Tools

### Read Tools (no keypair required)

#### get_balance

Get SOL balance for a Solana address.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | Yes | Solana address (base58, 32-44 chars) |
| `commitment` | string | No | Commitment level (default: `confirmed`) |

**Returns:** Balance in lamports (1 SOL = 1,000,000,000 lamports)

---

#### get_slot

Get current slot/block height.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `commitment` | string | No | Commitment level (default: `confirmed`) |

**Returns:** Current slot number

---

#### get_transaction

Fetch transaction details by signature.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `signature` | string | Yes | Transaction signature (base58, 87-88 chars) |
| `commitment` | string | No | Commitment level (default: `confirmed`) |

**Returns:** Full transaction data including status, fees, and account changes

---

### Write Tools (requires keypair)

#### transfer_sol [IRREVERSIBLE]

Transfer SOL from configured wallet to recipient.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `to_address` | string | Yes | Recipient Solana address |
| `amount_lamports` | u64 | Yes | Amount in lamports (1 SOL = 1,000,000,000) |

**Returns:** Transaction signature, amount, addresses

**Note:** Transaction fee (~5000 lamports) deducted from sender.

---

#### transfer_token [IRREVERSIBLE]

Transfer SPL tokens (USDC, USDT, etc.) to recipient.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `to_address` | string | Yes | Recipient Solana address |
| `token_mint` | string | Yes | Token mint address |
| `amount` | f64 | Yes | Amount in UI units (e.g., 1.5 = 1.5 tokens) |
| `decimals` | u8 | Yes | Token decimals (6 for USDC, 9 for native SOL) |

**Returns:** Transaction signature, token mint, amount (raw and UI), ATA addresses

**Note:** Recipient must have an Associated Token Account (ATA) for this token.

## Commitment Levels

| Level | Latency | Description |
|-------|---------|-------------|
| `processed` | Fastest | Transaction included in block, may rollback |
| `confirmed` | ~400ms | Default. Block produced by supermajority |
| `finalized` | ~1s | Permanent, cannot be rolled back |

## Examples

### Get Balance

```json
{
  "tool": "get_balance",
  "arguments": {
    "address": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2",
    "commitment": "confirmed"
  }
}
```

### Transfer SOL

```json
{
  "tool": "transfer_sol",
  "arguments": {
    "to_address": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2",
    "amount_lamports": 1000000000
  }
}
```

### Transfer USDC (devnet)

```json
{
  "tool": "transfer_token",
  "arguments": {
    "to_address": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2",
    "token_mint": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
    "amount": 10.0,
    "decimals": 6
  }
}
```

## Security

### Mainnet Guard

Transfers on `mainnet` or custom networks require explicit risk acceptance:

```bash
# Environment variable
export SOLANA_ACCEPT_RISK=true

# Or CLI flag
solana-onchain-mcp --accept-risk
```

This prevents accidental mainnet transactions. Devnet and testnet do not require this.

### Keypair Best Practices

- **Restrict permissions:** `chmod 600 ~/.config/solana/id.json`
- **Never commit:** Add keypair paths to `.gitignore`
- **Test first:** Use devnet/testnet before mainnet
- **Backup securely:** Store keypair backups in encrypted storage

### Read-Only Mode

If `SOLANA_KEYPAIR_PATH` is unset or invalid:
- Server starts with warning (not error)
- Transfer tools are filtered from available tools
- Read operations work normally

### HTTP Mode Security (Planned)

HTTP mode has additional restrictions:
- Keypair disabled by default
- `--http-allow-keypair` requires:
  - `--accept-risk` flag
  - `--host 127.0.0.1` (localhost only)
- Warning: "Ensure reverse proxy with auth!"

## Troubleshooting

### Invalid Keypair

```
Warning: Invalid keypair path, running in read-only mode
```

Check that:
- Path exists and is readable
- File contains valid JSON array of 64 bytes

### Mainnet Risk Not Accepted

```
Error: Mainnet requires --accept-risk
```

Set `SOLANA_ACCEPT_RISK=true` or pass `--accept-risk` flag.

### Invalid Address

```
Error: InvalidAddress - must be 32-44 characters
```

Ensure address is valid base58-encoded Solana public key.

### RPC Errors

Common causes:
- Network connectivity issues
- Rate limiting (use custom RPC endpoint)
- Invalid network URL
