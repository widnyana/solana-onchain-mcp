---
status: resolved
priority: p2
issue_id: 006
tags: [security, information-disclosure, code-review]
dependencies: []
date: 2026-02-21
---

# RPC Error Messages May Leak Internal Information

## Problem Statement

RPC errors are passed through with full context, potentially exposing internal RPC URLs, node versions, or other sensitive infrastructure details.

## Findings

**Location**: `src/error.rs:43-53`

- Full RPC error exposed to users
- May contain URLs, node info, stack traces
- Information disclosure risk

```rust
impl From<solana_client::client_error::ClientError> for SolanaMcpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        let msg = match err.kind() {
            ClientErrorKind::RpcError(rpc_err) => {
                format!("RPC error: {}", rpc_err)  // Full error exposure
            }
            // ...
        };
        SolanaMcpError::RpcError(msg)
    }
}
```

## Proposed Solutions

### Option 1: Sanitize Error Messages (Recommended)
Log full error internally, return sanitized version.

```rust
ClientErrorKind::RpcError(rpc_err) => {
    tracing::warn!("RPC error: {}", rpc_err);  // Log internally
    "RPC request failed".to_string()           // Return sanitized
}
```

**Pros**: Prevents information disclosure
**Cons**: Less debugging info for users
**Effort**: Small
**Risk**: Low

### Option 2: Filter Specific Patterns
Remove known sensitive patterns (URLs, versions) while keeping context.

**Pros**: Balances security and debuggability
**Cons**: Complex pattern matching
**Effort**: Medium
**Risk**: Medium

## Recommended Action

Implement Option 1 with internal logging.

## Technical Details

- **Affected Files**: `src/error.rs`
- **Components**: Error handling
- **Security Impact**: Information disclosure prevention

## Acceptance Criteria

- [ ] Add tracing dependency if not present
- [ ] Log full RPC errors internally
- [ ] Return sanitized error messages
- [ ] Test error paths

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security review |

## Resources

- OWASP Information Disclosure
- Related: `src/error.rs`
