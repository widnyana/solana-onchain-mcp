---
status: pending
priority: p3
issue_id: 026
tags: [documentation, maintainability, code-review]
dependencies: []
date: 2026-02-21
---

# Constants Lack Documentation

## Problem Statement

The newly added constants `MAX_PROGRAM_ACCOUNTS` and `MAX_TRANSACTION_INPUT_SIZE` have values without documented rationale.

## Findings

**Location**: `src/rpc/client.rs:32-33`

```rust
const MAX_PROGRAM_ACCOUNTS: usize = 10_000;
const MAX_TRANSACTION_INPUT_SIZE: usize = 4096;
```

**Problems**:
1. Why 10,000 accounts? (Memory limit? RPC limit?)
2. Why 4KB for transaction input? (Base64 overhead? Protocol limit?)
3. Future maintainers cannot make informed decisions about changing these

**Impact**: Low - maintainability concern, no runtime impact.

## Proposed Solutions

### Option 1: Add Documentation Comments (Recommended)
Add `///` doc comments explaining limits.

```rust
/// Maximum accounts returned by get_program_accounts to prevent memory exhaustion.
/// Solana RPC can return unbounded results; this limit protects against OOM.
const MAX_PROGRAM_ACCOUNTS: usize = 10_000;

/// Maximum base64-encoded transaction input size (~4KB covers base64 overhead).
/// Solana protocol limit is ~1.2KB raw; this allows for encoding overhead.
const MAX_TRANSACTION_INPUT_SIZE: usize = 4096;
```

**Pros**: Zero runtime cost, high clarity value
**Cons**: None
**Effort**: Small
**Risk**: None

## Recommended Action

Add documentation comments to both constants.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Components**: Documentation

## Acceptance Criteria

- [ ] Add doc comments to MAX_PROGRAM_ACCOUNTS
- [ ] Add doc comments to MAX_TRANSACTION_INPUT_SIZE
- [ ] Verify cargo doc includes documentation

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture + Simplicity review |

## Resources

- Rust documentation best practices
