---
status: pending
priority: p3
issue_id: 012
tags: [code-quality, style, code-review]
dependencies: []
date: 2026-02-21
---

# Inconsistent Derive Macro Formatting

## Problem Statement

Mixed styles for serde derive macros across files. Some use `::serde::Deserialize` (leading `::`), others use `serde::Deserialize`. This is visual noise with no semantic difference.

## Findings

**Locations**: Multiple tool files

```rust
// Style A (get_account_info.rs, get_signatures_for_address.rs):
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]

// Style B (get_token_accounts_by_owner.rs, simulate_transaction.rs):
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
```

## Proposed Solutions

### Option 1: Standardize to No Leading :: (Recommended)
Use `serde::Deserialize` consistently.

**Pros**: Cleaner, more common style
**Cons**: None
**Effort**: Small
**Risk**: None

## Recommended Action

Run search-replace to standardize all derive macros.

## Technical Details

- **Affected Files**: All files in `src/tools/`
- **Components**: Style consistency

## Acceptance Criteria

- [ ] Standardize all derive macros to `serde::`
- [ ] Verify compilation
- [ ] No functional changes

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Code simplicity review |

## Resources

- Rust style guide
