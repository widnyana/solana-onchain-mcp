---
status: pending
priority: p2
issue_id: 023
tags: [security, ux, error-handling, code-review]
dependencies: []
date: 2026-02-21
---

# Sanitization Patterns Are Overly Broad

## Problem Statement

The `sanitize_rpc_message` function uses overly broad keywords that redact legitimate, non-sensitive error messages, making debugging difficult.

## Findings

**Location**: `src/error.rs:47-56`

```rust
let sensitive_patterns = [
    "private", "secret", "key", "password", "token", "credential", "signature", "account",
];
```

**Problems**:
1. `"account"` matches normal Solana errors like "account not found" or "insufficient account balance"
2. `"token"` matches SPL token errors unrelated to authentication tokens
3. `"key"` could match "key not found" or other non-sensitive contexts
4. `"signature"` matches legitimate transaction signature errors

**Impact**: Users get redacted errors when no actual sensitive data is present, making debugging impossible.

Example:
```
// Original error: "Account 7xKX... not found"
// Sanitized: "RPC request failed (details redacted for security)"
// User sees no useful information
```

## Proposed Solutions

### Option 1: Use More Specific Patterns (Recommended)
Match against more specific patterns with context.

```rust
let sensitive_patterns = [
    "private key",
    "secret key",
    "password:",
    "bearer ",
    "api key",
];
```

**Pros**: Reduces false positives
**Cons**: May miss some sensitive data
**Effort**: Small
**Risk**: Low

### Option 2: Remove Pattern Matching, Just Truncate
Only truncate long messages without pattern matching.

```rust
fn sanitize_rpc_message(msg: &str) -> String {
    const MAX_LEN: usize = 200;
    if msg.len() > MAX_LEN {
        format!("{}...", &msg[..MAX_LEN])
    } else {
        msg.to_string()
    }
}
```

**Pros**: Simple, no false positives
**Cons**: No security filtering
**Effort**: Small
**Risk**: Medium

## Recommended Action

Implement Option 1 - use more specific patterns. Balance between security and usability.

## Technical Details

- **Affected Files**: `src/error.rs`
- **Components**: Error handling, security

## Acceptance Criteria

- [ ] Refine pattern list to be more specific
- [ ] Test with common Solana error messages
- [ ] Verify legitimate errors are not redacted
- [ ] Document pattern rationale

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security review |

## Resources

- Related: src/error.rs
