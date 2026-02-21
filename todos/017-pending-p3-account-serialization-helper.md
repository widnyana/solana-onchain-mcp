---
status: pending
priority: p3
issue_id: 017
tags: [refactor, dry, code-review]
dependencies: []
date: 2026-02-21
---

# DRY: Extract Account Serialization Helper

## Problem Statement

Account JSON serialization is duplicated across 3 methods in `client.rs`. Same pattern appears in `get_account_info`, `get_multiple_accounts`, and `get_program_accounts`.

## Findings

**Location**: `src/rpc/client.rs` (lines 153-163, 197-207, 329-345)

Duplicated pattern:
```rust
serde_json::json!({
    "lamports": acc.lamports,
    "owner": acc.owner.to_string(),
    "executable": acc.executable,
    "rent_epoch": acc.rent_epoch,
    "data": data_encoded,
    "space": acc.data.len(),
})
```

**Impact**: ~30 lines of duplicated code, single point of failure for format changes.

## Proposed Solutions

### Option 1: Extract Helper Method (Recommended)

```rust
impl SolanaRpcClient {
    fn serialize_account(account: &Account, encoding: UiAccountEncoding) -> serde_json::Value {
        let data_encoded = Self::encode_account_data(&account.data, encoding);
        serde_json::json!({
            "lamports": account.lamports,
            "owner": account.owner.to_string(),
            "executable": account.executable,
            "rent_epoch": account.rent_epoch,
            "data": data_encoded,
            "space": account.data.len(),
        })
    }
}
```

**Pros**: DRY, single point of change
**Cons**: None
**Effort**: Small
**Risk**: None

## Recommended Action

Extract the helper method and use it in all 3 locations.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Methods to update**: `get_account_info`, `get_multiple_accounts`, `get_program_accounts`
- **LOC reduction**: ~20-30 lines

## Acceptance Criteria

- [ ] Create `serialize_account` helper
- [ ] Update `get_account_info` to use helper
- [ ] Update `get_multiple_accounts` to use helper
- [ ] Update `get_program_accounts` to use helper
- [ ] Run `cargo check` - no errors
- [ ] Run tests - all pass

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Code simplicity review |

## Resources

- DRY principle
