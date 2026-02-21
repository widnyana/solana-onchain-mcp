---
status: pending
priority: p2
issue_id: 015
tags: [architecture, async, performance, code-review]
dependencies: []
date: 2026-02-21
---

# Inconsistent Sync/Async Model for RPC Methods

## Problem Statement

Read methods are synchronous while write methods are async with `spawn_blocking`. This inconsistency could cause the async runtime to be blocked under load.

## Findings

**Location**: `src/rpc/client.rs`

**Read methods (sync, blocking):**
```rust
pub fn get_account_info(&self, ...) -> Result<Option<serde_json::Value>>
pub fn get_program_accounts(&self, ...) -> Result<serde_json::Value>
// etc.
```

**Write methods (async with spawn_blocking):**
```rust
pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature> {
    spawn_blocking(move || {
        client.client.send_transaction_with_config(&tx, ...)
    }).await...
}
```

**Risk**: Under concurrent load, synchronous read methods could block the Tokio runtime executor, starving other async tasks.

## Proposed Solutions

### Option 1: Make All RPC Methods Async (Recommended)
Wrap sync methods with `spawn_blocking` like write methods.

```rust
pub async fn get_account_info(&self, ...) -> Result<Option<serde_json::Value>> {
    let client = self.clone();
    spawn_blocking(move || {
        // existing implementation
    }).await.map_err(|_| SolanaMcpError::TaskJoinError)?
}
```

**Pros**: Consistent async model, non-blocking under load
**Cons**: Breaking API change for handler.rs
**Effort**: Medium
**Risk**: Low

### Option 2: Document Current Behavior
Add documentation noting the sync/async split and its implications.

**Pros**: No code changes
**Cons**: Doesn't address the actual risk
**Effort**: Trivial
**Risk**: None

## Recommended Action

Implement Option 1 for consistency. The handler already runs in async context, so this fits naturally.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, `src/handler.rs`
- **Components**: All RPC methods
- **Breaking Change**: Yes - method signatures change

## Acceptance Criteria

- [ ] Convert all read methods to async
- [ ] Wrap sync RPC calls in `spawn_blocking`
- [ ] Update handler.rs call sites
- [ ] Run `cargo check` - no errors
- [ ] Run tests

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Tokio docs on `spawn_blocking`
- Related: Write methods already use this pattern
