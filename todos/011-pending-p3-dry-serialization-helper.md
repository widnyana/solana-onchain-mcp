---
status: pending
priority: p3
issue_id: 011
tags: [code-quality, dry, refactor, code-review]
dependencies: []
date: 2026-02-21
---

# DRY: Duplicated Account JSON Construction

## Problem Statement

Account JSON construction is duplicated across 3 methods in `client.rs`. Same structure repeated for `get_account_info`, `get_multiple_accounts`, and `get_program_accounts`.

## Findings

**Location**: `src/rpc/client.rs:154-162, 198-206, 332-343`

```rust
// Same pattern repeated 3 times:
serde_json::json!({
    "lamports": acc.lamports,
    "owner": acc.owner.to_string(),
    "executable": acc.executable,
    "rent_epoch": acc.rent_epoch,
    "data": data_encoded,
    "space": acc.data.len(),
})
```

## Proposed Solutions

### Option 1: Extract Helper Function (Recommended)
Create `account_to_json` helper.

```rust
fn account_to_json(account: Account, encoding: UiAccountEncoding) -> serde_json::Value {
    serde_json::json!({
        "lamports": account.lamports,
        "owner": account.owner.to_string(),
        "executable": account.executable,
        "rent_epoch": account.rent_epoch,
        "data": Self::encode_account_data(&account.data, encoding),
        "space": account.data.len(),
    })
}
```

**Pros**: Single source of truth, easy to modify
**Cons**: Minor refactoring
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1 - extract to helper function.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Components**: Account serialization

## Acceptance Criteria

- [ ] Create `account_to_json` helper
- [ ] Replace all 3 duplicated patterns
- [ ] Verify tests pass
- [ ] Verify JSON output unchanged

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Related: `encode_account_data` helper
