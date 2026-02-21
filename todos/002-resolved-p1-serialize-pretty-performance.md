---
status: resolved
priority: p1
issue_id: 002
tags: [performance, code-quality, code-review]
dependencies: []
date: 2026-02-21
---

# to_string_pretty() Performance Overhead

## Problem Statement

All 11 tool handlers use `serde_json::to_string_pretty()` which adds ~30-40% size overhead and is ~50-100% slower than `to_string()`. This impacts every single RPC response.

## Findings

**Locations**: All files in `src/tools/`

- `get_program_accounts.rs:60`
- `get_signatures_for_address.rs:49`
- `get_multiple_accounts.rs:52`
- `simulate_transaction.rs:47`
- `get_account_info.rs:42`
- `get_token_accounts_by_owner.rs:61`
- `get_transaction.rs:30`
- `get_balance.rs:42`
- `get_slot.rs:30`
- `handler.rs:117, 129` (transfer tools)

```rust
// Appears in EVERY tool handler:
serde_json::to_string_pretty(&result).unwrap_or_else(|_| "...".to_string())
```

## Proposed Solutions

### Option 1: Replace with to_string() (Recommended)
Replace all `to_string_pretty()` with `to_string()`.

```rust
serde_json::to_string(&result).expect("RPC response serialization")
```

**Pros**: 50% faster, 30% smaller output
**Cons**: Less human-readable (but agents don't need pretty)
**Effort**: Small
**Risk**: Low

### Option 2: Extract Helper Function
Create a shared helper to avoid code duplication.

```rust
// src/tools/mod.rs
pub fn json_to_text<T: serde::Serialize>(value: &T) -> TextContent {
    TextContent::from(
        serde_json::to_string(value).expect("RPC response serialization")
    )
}
```

**Pros**: DRY, single point of change
**Cons**: Slightly more refactoring
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 2 (Helper Function) - addresses both performance AND code duplication.

## Technical Details

- **Affected Files**: All files in `src/tools/`, `src/handler.rs`
- **Components**: All MCP tool handlers
- **Metrics**: ~50% faster serialization, ~30% smaller output

## Acceptance Criteria

- [ ] Create `json_to_text()` helper in `src/tools/mod.rs`
- [ ] Replace all `to_string_pretty()` calls with helper
- [ ] Update fallback pattern to use `expect()` instead of `unwrap_or_else()`
- [ ] Verify tests pass
- [ ] Benchmark response size reduction

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Performance review |

## Resources

- serde_json performance docs
- Related: P2-003 (DRY serialization pattern)
