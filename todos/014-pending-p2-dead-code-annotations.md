---
status: pending
priority: p2
issue_id: 014
tags: [quality, dead-code, code-review]
dependencies: []
date: 2026-02-21
---

# error.rs: False Dead Code Annotations

## Problem Statement

Two error variants in `error.rs` are marked with `#[allow(dead_code)]` but they ARE actually used in the codebase. This violates the project's rust.md rule against silencing dead code warnings.

## Findings

**Location**: `src/error.rs:33-38`

```rust
#[error("Transaction failed: {0}")]
#[allow(dead_code)]
TransactionFailed(String),

#[error("Invalid token account: {0}")]
#[allow(dead_code)]
InvalidTokenAccount(String),
```

**Actual Usage**:
- `TransactionFailed` - used in `src/tools/transfer_token.rs:72`
- `InvalidTokenAccount` - used in `src/tools/transfer_token.rs:32`

The warnings likely come from the variants not being constructed in the same module, not from being unused.

## Proposed Solutions

### Option 1: Remove Annotations (Recommended)
Simply remove the `#[allow(dead_code)]` attributes.

**Pros**: Follows project rules, accurate reflection of usage
**Cons**: None
**Effort**: Trivial
**Risk**: None

### Option 2: Investigate Root Cause
Find why Rust thinks these are dead code and fix the underlying issue.

**Pros**: Addresses root cause
**Cons**: May be false positive from macro usage
**Effort**: Small
**Risk**: Low

## Recommended Action

Remove the `#[allow(dead_code)]` attributes immediately. If compiler warnings appear, investigate the actual cause.

## Technical Details

- **Affected Files**: `src/error.rs`
- **Components**: Error handling
- **Related**: Project rust.md rules

## Acceptance Criteria

- [ ] Remove `#[allow(dead_code)]` from `TransactionFailed`
- [ ] Remove `#[allow(dead_code)]` from `InvalidTokenAccount`
- [ ] Run `cargo check` - no new warnings
- [ ] Verify variants compile correctly

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Code simplicity review |

## Resources

- Project rules: `.claude/rules/rust.md`
