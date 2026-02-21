---
status: resolved
priority: p2
issue_id: 005
tags: [security, dos, code-review]
dependencies: []
date: 2026-02-21
---

# simulate_transaction: Missing Transaction Size Validation

## Problem Statement

The `simulate_transaction` function decodes arbitrary base64/base58 input without size validation. Maliciously large payloads could cause memory exhaustion.

## Findings

**Location**: `src/rpc/client.rs:357-367`

- No validation of input size before decoding
- Base64 decode allocates memory proportional to input
- bincode deserialize then processes unbounded bytes

```rust
let tx_bytes = match encoding.unwrap_or("base64") {
    "base58" => bs58::decode(transaction).into_vec()?,  // No size check
    _ => BASE64_STANDARD.decode(transaction)?,           // No size check
};
```

## Proposed Solutions

### Option 1: Add Size Limit (Recommended)
Add maximum transaction size validation before decoding.

```rust
const MAX_TRANSACTION_SIZE: usize = 2048; // Solana transactions max ~1232 bytes

if transaction.len() > MAX_TRANSACTION_SIZE * 2 { // Base64 is ~4/3x original
    return Err(SolanaMcpError::RpcError("Transaction too large".to_string()));
}
```

**Pros**: Simple, prevents DoS
**Cons**: May reject legitimate edge cases
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1 with reasonable buffer for encoding overhead.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Components**: Transaction simulation
- **Security Impact**: DoS prevention

## Acceptance Criteria

- [ ] Add `MAX_TRANSACTION_INPUT_SIZE` constant
- [ ] Validate size before decoding
- [ ] Return clear error when exceeded
- [ ] Add test for oversized input

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security review |

## Resources

- Solana transaction size limits
- Related: `src/tools/simulate_transaction.rs`
