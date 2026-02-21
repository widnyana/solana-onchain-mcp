---
status: resolved
priority: p2
issue_id: 007
tags: [bug, api, code-review]
dependencies: []
date: 2026-02-21
---

# get_token_accounts_by_owner: Unused Commitment Parameter

## Problem Statement

The `get_token_accounts_by_owner` RPC method accepts a `commitment` parameter but never uses it (prefixed with `_`). API contract doesn't match implementation.

## Findings

**Location**: `src/rpc/client.rs:217`

```rust
pub fn get_token_accounts_by_owner(
    &self,
    owner_address: &str,
    mint: Option<&str>,
    program_id: Option<&str>,
    _commitment: Option<&str>,  // <-- Unused
) -> Result<serde_json::Value>
```

## Proposed Solutions

### Option 1: Implement Commitment Support (Recommended)
Wire the commitment parameter through to the RPC call.

```rust
let config = RpcAccountInfoConfig {
    commitment: Some(Self::parse_commitment(commitment)),
    // ...
};
```

**Pros**: API works as documented
**Cons**: Requires finding correct config struct
**Effort**: Small
**Risk**: Low

### Option 2: Remove Parameter
Remove from both tool and client method signatures.

**Pros**: API matches implementation
**Cons**: Less flexibility
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1 - add commitment support for consistency with other methods.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, `src/tools/get_token_accounts_by_owner.rs`
- **Components**: Token account queries

## Acceptance Criteria

- [ ] Remove underscore prefix from parameter
- [ ] Wire commitment to RPC config
- [ ] Test with different commitment levels
- [ ] Verify tool behavior

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Related: Other methods using commitment pattern
