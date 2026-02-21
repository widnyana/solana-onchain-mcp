---
status: resolved
priority: p1
issue_id: 001
tags: [security, performance, memory, code-review]
dependencies: []
date: 2026-02-21
---

# get_program_accounts: Unbounded Memory Allocation

## Problem Statement

The `get_program_accounts` tool can return massive result sets without any programmatic limits, potentially causing memory exhaustion (OOM). Popular programs like the Token Program have millions of accounts.

## Findings

**Location**: `src/rpc/client.rs:329-345`

- Popular programs (Token Program, Metaplex) can have 100K+ accounts
- Each account includes ~100-500 bytes of data + metadata
- Estimated: 100K accounts x 300 bytes = 30MB+ in memory per request
- At 1M accounts (possible for Token Program) = 300MB+ per request
- No pagination = client cannot stream results

```rust
// Current: Collects ALL accounts into memory
let result: Vec<serde_json::Value> = accounts
    .into_iter()
    .map(|(pubkey, account)| { ... })
    .collect();
```

## Proposed Solutions

### Option 1: Hard Limit with Error (Recommended)
Add a hard limit on result count with helpful error message.

```rust
const MAX_PROGRAM_ACCOUNTS: usize = 10_000;

let accounts = self.client.get_program_accounts_with_config(&program, config)?;
if accounts.len() > MAX_PROGRAM_ACCOUNTS {
    return Err(SolanaMcpError::RpcError(
        format!("Too many accounts ({}). Add filters (data_size, memcmp) to narrow results.", accounts.len())
    ));
}
```

**Pros**: Simple, explicit, provides guidance
**Cons**: May reject legitimate large queries
**Effort**: Small
**Risk**: Low

### Option 2: Add Pagination Support
Implement cursor-based pagination.

**Pros**: Handles any size result set
**Cons**: More complex, requires state management
**Effort**: Medium
**Risk**: Medium

## Recommended Action

Implement Option 1 (Hard Limit) immediately. Add pagination in future iteration if needed.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, `src/tools/get_program_accounts.rs`
- **Components**: RPC client, get_program_accounts tool
- **Related**: P2-001 (get_signatures_for_address limit)

## Acceptance Criteria

- [ ] Add `MAX_PROGRAM_ACCOUNTS` constant (10,000)
- [ ] Return error when limit exceeded
- [ ] Error message includes filter guidance
- [ ] Update tool description to document limit
- [ ] Add test for limit enforcement

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security and performance review |

## Resources

- Solana RPC docs: getProgramAccounts
- Related: `src/tools/get_program_accounts.rs`
