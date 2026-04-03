# Usage Guide

## Installation

### Method 1: Prebuilt Binary (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/widnyana/solana-onchain-mcp/refs/heads/main/install.sh | bash
```

**Advantages:**
- Fast: No compilation required
- Cross-platform: Linux, macOS (ARM64 and AMD64)
- Stable: Uses official release binaries
- Small: ~3MB download

**Manual Download:**
Visit [Releases](https://github.com/widnyana/solana-onchain-mcp/releases) and download:
- `solana-onchain-mcp-linux-amd64.tar.gz`
- `solana-onchain-mcp-linux-arm64.tar.gz`
- `solana-onchain-mcp-darwin-amd64.tar.gz`
- `solana-onchain-mcp-darwin-arm64.tar.gz`

### Method 2: Install from crates.io

```bash
cargo install solana-onchain-mcp
```

### Method 3: Build from Source

```bash
cargo install --git https://github.com/widnyana/solana-onchain-mcp
```

**Requirements:**
- Rust 1.85+ (Edition 2024)
- OpenSSL development headers

**Note:** Compilation may take 5-10 minutes.

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SOLANA_NETWORK` | `devnet` | `mainnet`, `devnet`, `testnet`, or RPC URL |
| `SOLANA_KEYPAIR_PATH` | (none) | Path to keypair JSON (required for transfers) |
| `SOLANA_ACCEPT_RISK` | `false` | Set `true` for transfers on mainnet/custom |
| `RUST_LOG` | `info` | Log level |

### CLI Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--accept-risk` | false | Enable transfers on mainnet/custom |
| `--port` | 3000 | HTTP port |
| `--host` | 127.0.0.1 | HTTP host |
| `--http-allow-keypair` | false | Allow keypair in HTTP mode (requires `--accept-risk` and `--host 127.0.0.1`) |

### Networks

| Network | RPC URL |
|---------|---------|
| `mainnet` | `https://api.mainnet-beta.solana.com` |
| `devnet` | `https://api.devnet.solana.com` |
| `testnet` | `https://api.testnet.solana.com` |
| Custom | Any `http://` or `https://` URL |

Private IPs blocked for `https://`. Use `http://` for local RPC.

## Tools

### Read Tools

#### get_balance

SOL balance for an address.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | Yes | Solana address (base58) |
| `commitment` | string | No | `processed`, `confirmed` (default), `finalized` |

Returns: balance in lamports (1 SOL = 10^9 lamports)

#### get_account_info

Account data and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | Yes | Solana address |
| `encoding` | string | No | `base64` (default), `base58`, `jsonParsed` |
| `commitment` | string | No | Commitment level |

Returns: account data, owner, lamports, executable flag

#### get_multiple_accounts

Batch fetch up to 100 accounts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `addresses` | [string] | Yes | Array of addresses (max 100) |
| `encoding` | string | No | `base64` (default), `base58`, `jsonParsed` |
| `commitment` | string | No | Commitment level |

Returns: array of accounts (null for non-existent)

#### get_token_accounts_by_owner

SPL token accounts for a wallet.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `owner_address` | string | Yes | Wallet address |
| `mint` | string | No* | Filter by token mint |
| `program_id` | string | No* | Filter by token program |
| `commitment` | string | No | Commitment level |

*Must specify `mint` OR `program_id` (not both).

Common program IDs:
- `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` (Token)
- `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` (Token-2022)

#### get_program_accounts

Accounts owned by a program.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `program_id` | string | Yes | Program address |
| `data_size` | u64 | No* | Filter by data size |
| `memcmp.offset` | u64 | No* | Memcmp filter offset |
| `memcmp.bytes` | string | No* | Memcmp filter bytes (base58) |
| `encoding` | string | No | `base64` (default), `base58`, `jsonParsed` |
| `commitment` | string | No | Commitment level |

*At least one filter (`data_size` or `memcmp`) is required.

#### get_transaction

Transaction details by signature.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `signature` | string | Yes | Transaction signature (base58) |
| `commitment` | string | No | Commitment level |

Returns: full transaction with meta, status, fees

#### get_signatures_for_address

Transaction history for an address.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | Yes | Address to query |
| `limit` | u64 | No | Max results (1-1000, default 100) |
| `before` | string | No | Paginate from this signature |
| `until` | string | No | Paginate until this signature |
| `commitment` | string | No | Commitment level |

Returns: signatures with slot, blockTime, err, confirmationStatus

#### get_signature_status

Check if transaction is confirmed.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `signature` | string | Yes | Transaction signature |

Returns: `{ confirmed: bool, slot: number | null }`

#### get_slot

Current slot/block height.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `commitment` | string | No | Commitment level |

#### get_server_info

Server configuration and network information.

No parameters required.

Returns: network type, RPC URL, keypair status, accept_risk setting

#### simulate_transaction

Test transaction without signing.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `transaction` | string | Yes | Base64 serialized transaction |
| `encoding` | string | No | `base64` (default), `base58` |
| `replace_recent_blockhash` | bool | No | Use current blockhash |
| `commitment` | string | No | Commitment level |

Returns: logs, compute units, errors

#### inspect_transaction_raw

Raw transaction with program name annotations.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `signature` | string | Yes | Transaction signature |
| `commitment` | string | No | Commitment level |

Returns: full transaction JSON with `programName` and `error_interpretation` fields

#### inspect_transaction_humanized

Human-readable transaction summary.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `signature` | string | Yes | Transaction signature |
| `commitment` | string | No | Commitment level |

Returns: status, fee, summary, instructions with explanations, accounts

#### query_transactions

Filter and classify transactions for a wallet with human-readable details and cursor-based pagination.
Use this instead of `get_signatures_for_address` when you need full transaction details, type filtering, or summaries.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `address` | string | No | Wallet address. Omit to use server keypair address |
| `cursor` | string | No | Pagination cursor from previous response. Omit for first request |
| `since_days` | u64 | No | Days back from now (mutually exclusive with `after_timestamp`) |
| `before_timestamp` | i64 | No | Unix epoch upper bound (exclusive) |
| `after_timestamp` | i64 | No | Unix epoch lower bound (exclusive, mutually exclusive with `since_days`) |
| `tx_types` | [string] | No | Filter by type: `transfer`, `swap`, `mint`, `burn`, `nft`, `unknown`. Omit for all |
| `include_failed` | bool | No | Include failed transactions (default: false) |
| `limit` | u64 | No | Max matched results (default: 20, max: 1000) |
| `compact` | bool | No | Compact output (default: true). Set `false` for full humanized JSON |
| `commitment` | string | No | Commitment level |

**Pagination:** Omit `cursor` for the first request. Pass `nextCursor` from the response as `cursor` to fetch the next page. No `nextCursor` in response means end of results. Time filters cannot be combined with `cursor`.

Returns: matched transactions with type, status, fee, summary; plus `result_summary` (type counts, total fees, failed count) and optional `nextCursor`.

### Write Tools

#### transfer_sol

Transfer SOL to recipient.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `to_address` | string | Yes | Recipient address |
| `amount_lamports` | u64 | Yes | Amount in lamports |

Fee: ~5000 lamports deducted from sender.

#### transfer_token

Transfer SPL tokens.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `to_address` | string | Yes | Recipient address |
| `token_mint` | string | Yes | Token mint address |
| `amount` | f64 | Yes | Amount in UI units (e.g., 1.5) |
| `decimals` | u8 | Yes | Token decimals (6 for USDC) |

Recipient must have an ATA for this token.

#### create_associated_token_account

Create token account for a mint.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `token_mint` | string | Yes | Token mint address |
| `owner` | string | No | Owner (defaults to your wallet) |

Costs rent-exempt balance in SOL.

#### approve_token

Approve a delegate to spend tokens from your account.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `token_account` | string | Yes | Token account address |
| `delegate` | string | Yes | Delegate address to approve |
| `amount` | f64 | Yes | Amount in UI units (e.g., 1.5) |
| `decimals` | u8 | Yes | Token decimals (6 for USDC) |
| `owner` | string | No | Owner (defaults to your wallet) |

Returns: transaction signature

#### revoke_token

Revoke token delegate authority.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `token_account` | string | Yes | Token account address |
| `owner` | string | No | Owner (defaults to your wallet) |

Returns: transaction signature

#### close_token_account

Close an unused token account to reclaim rent.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `token_account` | string | Yes | Token account address |
| `owner` | string | No | Owner (defaults to your wallet) |

Returns: transaction signature and recovered lamports

## Commitment Levels

| Level | Latency | Description |
|-------|---------|-------------|
| `processed` | Fastest | In block, may rollback |
| `confirmed` | ~400ms | Default. Supermajority produced |
| `finalized` | ~1s | Permanent |

## Security

### Mainnet Guard

Transfers on `mainnet` or custom networks require:

```bash
export SOLANA_ACCEPT_RISK=true
# or
solana-onchain-mcp --accept-risk
```

Devnet/testnet don't require this.

### HTTP Mode

The server runs exclusively in HTTP mode using SSE (Server-Sent Events).

**Use Cases:**
- Running MCP server on a separate machine
- Sharing MCP server across multiple AI clients
- Integrating with web-based AI interfaces

**Read-Only Mode (Default):**
```bash
solana-onchain-mcp --port 3000
```

**With Keypair (Localhost Only):**
```bash
solana-onchain-mcp --http-allow-keypair --accept-risk --host 127.0.0.1 --port 3000
```

**Security Requirements:**
- `--http-allow-keypair` requires:
  - `--accept-risk` flag
  - `--host 127.0.0.1` (localhost only, rejected for other hosts)
- By default, the server runs read-only (keypair disabled)

**Example for AI Integration:**
```json
{
  "mcpServers": {
    "solana": {
      "url": "http://localhost:3000/mcp"
    }
  }
}
```

### Mainnet Usage

⚠️ **WARNING:** Mainnet operations involve real, irreversible transactions.

**Prerequisites:**
1. Test everything on devnet first
2. Use a dedicated wallet with minimal funds
3. Verify all transaction parameters
4. Understand gas fees and rent costs

**Enabling Mainnet:**
```bash
# Environment variable
export SOLANA_NETWORK=mainnet
export SOLANA_ACCEPT_RISK=true
export SOLANA_KEYPAIR_PATH=/path/to/keypair.json

# OR CLI flags
SOLANA_NETWORK=mainnet solana-onchain-mcp --accept-risk
```

**What Gets Warned:**
- When `SOLANA_ACCEPT_RISK=true` is set
- When network type is mainnet or custom URL
- When `--accept-risk` CLI flag is used
- Before server starts on mainnet
- When keypair loads on mainnet

**Best Practices:**
- Never use your main wallet
- Start with small test transactions
- Monitor logs for all warnings
- Keep backups of important keypairs
- Consider using a hardware wallet for large amounts

### Keypair Safety

- `chmod 600` on keypair file
- Add to `.gitignore`
- Test on devnet first
- Backup securely

### Read-Only Mode

If `SOLANA_KEYPAIR_PATH` is unset or invalid:
- Server starts with warning
- Write tools hidden
- Read tools work normally

## Troubleshooting

### Invalid Keypair

```
Warning: Invalid keypair path, running in read-only mode
```

Check: path exists, file is valid JSON array of 64 bytes.

### Mainnet Risk Not Accepted

```
Error: Mainnet requires --accept-risk
```

Set `SOLANA_ACCEPT_RISK=true` or `--accept-risk`.

### Invalid Address

```
Error: InvalidAddress - must be 32-44 characters
```

Use valid base58 Solana public key.

### RPC Errors

Common causes:
- Network issues
- Rate limiting (use custom RPC)
- Invalid URL
