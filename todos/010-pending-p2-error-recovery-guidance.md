---
status: pending
priority: p2
issue_id: 010
tags: [agent-native, usability, error-handling, code-review]
dependencies: []
date: 2026-02-21
---

# Error Messages Lack Agent Recovery Guidance

## Problem Statement

Error messages are descriptive but lack recovery hints. Agents cannot self-correct from errors without understanding what to fix.

## Findings

**Location**: `src/error.rs:4-39`

- `InvalidAddress` tells what's wrong, not what format is expected
- `RpcError` just wraps the message without suggesting common fixes
- No guidance on how to resolve errors

```rust
// Current:
InvalidAddress(String),  // "Invalid address 'foo'"

// Better:
InvalidAddress(String),  // "Invalid address 'foo': expected base58-encoded public key (32-44 chars)"
```

## Proposed Solutions

### Option 1: Enrich Error Messages (Recommended)
Add recovery guidance to error variants.

```rust
InvalidAddress(addr) => {
    format!("Invalid address '{}': expected base58-encoded Solana address (32-44 characters like '7xKX...')", addr)
}
```

**Pros**: Self-correcting agents
**Cons**: More verbose errors
**Effort**: Small
**Risk**: Low

### Option 2: Add Error Documentation
Create separate error documentation file with recovery steps.

**Pros**: Detailed guidance
**Cons**: Not inline, harder to access
**Effort**: Medium
**Risk**: Low

## Recommended Action

Implement Option 1 - enrich error messages with guidance inline.

## Technical Details

- **Affected Files**: `src/error.rs`
- **Components**: All error variants

## Acceptance Criteria

- [ ] Review all error variants
- [ ] Add format examples where applicable
- [ ] Add common fix suggestions
- [ ] Test error messages are helpful

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- Related: `src/error.rs`
