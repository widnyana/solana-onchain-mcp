---
status: pending
priority: p1
issue_id: 020
tags: [security, memory, performance, code-review]
dependencies: []
date: 2026-02-21
---

# get_program_accounts Limit Check Happens After Fetch

## Problem Statement

The `MAX_PROGRAM_ACCOUNTS` limit check occurs AFTER the RPC call completes, meaning the full response is already in memory before validation. This defeats the purpose of the memory protection.

## Findings

**Location**: `src/rpc/client.rs:333-343`

```rust
// Current flow:
let accounts = self.client.get_program_accounts_with_config(&program, config)?;
// ^ Network transfer: ALL accounts
// ^ Deserialization: ALL accounts into memory
// ^ Memory allocation: Vec with potentially 100,000+ accounts

if accounts.len() > Self::MAX_PROGRAM_ACCOUNTS {
    return Err(...);  // Too late - resources already consumed
}
```

**Impact**:
- An attacker can cause memory exhaustion by querying programs with many accounts
- 100,000 accounts = ~50MB memory already consumed before limit check
- Network bandwidth wasted on responses that will be rejected

## Proposed Solutions

### Option 1: Require Filters (Recommended)
Require callers to provide at least one filter (data_size or memcmp) to narrow results.

```rust
if data_size.is_none() && memcmp.is_none() {
    return Err(SolanaMcpError::RpcError(
        "get_program_accounts requires filters (data_size or memcmp) to prevent resource exhaustion".into()
    ));
}
```

**Pros**: Prevents unbounded queries entirely
**Cons**: Breaking change for callers without filters
**Effort**: Small
**Risk**: Low

### Option 2: Use data_slice to Limit Response
Use `data_slice: Some((0, 0))` to only return metadata, not full data.

**Pros**: No breaking change
**Cons**: Loses actual account data
**Effort**: Small
**Risk**: Medium

## Recommended Action

Implement Option 1 - require filters. This provides strong protection and matches the documented best practice.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, `src/tools/get_program_accounts.rs`
- **Components**: Memory protection, input validation

## Acceptance Criteria

- [ ] Add filter requirement validation before RPC call
- [ ] Update tool description to explain filter requirement
- [ ] Add tests for validation
- [ ] Verify memory protection works before fetch

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Performance review |

## Resources

- Related: 001-pending-p1-unbounded-memory-get-program-accounts.md (original fix)
