---
status: pending
priority: p2
issue_id: 016
tags: [defensive, validation, code-review]
dependencies: []
date: 2026-02-21
---

# get_multiple_accounts: Missing Client-Layer Limit

## Problem Statement

The 100-address limit is only enforced in the tool layer, not the client layer. Direct client usage bypasses this protection.

## Findings

**Tool layer** (`src/tools/get_multiple_accounts.rs:37-41`):
```rust
if self.addresses.len() > 100 {
    return Ok(CallToolResult::text_content(vec![TextContent::from(
        "Error: Maximum 100 addresses allowed per request",
    )]));
}
```

**Client layer** (`src/rpc/client.rs:166-210`):
```rust
pub fn get_multiple_accounts(&self, addresses: &[String], ...) -> Result<...> {
    // No limit check here!
    let pubkeys: Vec<Pubkey> = addresses.iter().map(...).collect()?;
    ...
}
```

**Risk**: If someone uses the client directly (tests, future code), the limit is bypassed.

## Proposed Solutions

### Option 1: Add Defensive Limit in Client (Recommended)
Add the limit check at the client layer.

```rust
const MAX_MULTIPLE_ACCOUNTS: usize = 100;

pub fn get_multiple_accounts(&self, addresses: &[String], ...) -> Result<...> {
    if addresses.len() > MAX_MULTIPLE_ACCOUNTS {
        return Err(SolanaMcpError::InvalidRequest(
            format!("Maximum {} addresses allowed", MAX_MULTIPLE_ACCOUNTS)
        ));
    }
    ...
}
```

**Pros**: Defensive programming, single source of truth
**Cons**: Slight duplication
**Effort**: Small
**Risk**: None

### Option 2: Remove Tool-Layer Check
Rely only on client-layer check.

**Pros**: Single validation point
**Cons**: Less informative error format in tool
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1. Keep tool-layer check for user-friendly error format, add client-layer for defense in depth.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Components**: `get_multiple_accounts` method
- **Constants**: Add `MAX_MULTIPLE_ACCOUNTS`

## Acceptance Criteria

- [ ] Add `MAX_MULTIPLE_ACCOUNTS` constant
- [ ] Add limit check in client method
- [ ] Return proper error type
- [ ] Tool-layer check can remain for UX

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security review |

## Resources

- Related: 001 (get_program_accounts limit)
