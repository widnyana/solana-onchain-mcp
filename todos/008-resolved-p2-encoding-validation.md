---
status: resolved
priority: p2
issue_id: 008
tags: [validation, api, code-review]
dependencies: []
date: 2026-02-21
---

# Encoding Parameter Silently Defaults to Base64

## Problem Statement

The `parse_encoding` function defaults to `Base64` for unknown encodings rather than rejecting them. This could mask user errors or lead to unexpected behavior.

## Findings

**Location**: `src/rpc/client.rs:50-56`

```rust
fn parse_encoding(encoding: &str) -> UiAccountEncoding {
    match encoding {
        "base58" => UiAccountEncoding::Base58,
        "jsonParsed" => UiAccountEncoding::JsonParsed,
        _ => UiAccountEncoding::Base64,  // Silently defaults for invalid input
    }
}
```

## Proposed Solutions

### Option 1: Return Error for Invalid Encodings (Recommended)
Make encoding validation explicit.

```rust
fn parse_encoding(encoding: &str) -> Result<UiAccountEncoding> {
    match encoding {
        "base64" | "base64+zstd" => Ok(UiAccountEncoding::Base64),
        "base58" => Ok(UiAccountEncoding::Base58),
        "jsonParsed" => Ok(UiAccountEncoding::JsonParsed),
        _ => Err(SolanaMcpError::RpcError(format!("Invalid encoding: {}", encoding))),
    }
}
```

**Pros**: Explicit validation, catches typos
**Cons**: Breaking change for callers
**Effort**: Medium
**Risk**: Low

### Option 2: Log Warning When Defaulting
Keep current behavior but log a warning.

```rust
_ => {
    tracing::warn!("Invalid encoding '{}', defaulting to base64", encoding);
    UiAccountEncoding::Base64
}
```

**Pros**: No breaking change
**Cons**: Still allows invalid input
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 2 for immediate improvement, consider Option 1 for next major version.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`
- **Components**: Encoding parsing

## Acceptance Criteria

- [ ] Add warning log for invalid encoding
- [ ] Document valid encoding options in tool descriptions
- [ ] Test with invalid encoding

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security review |

## Resources

- Solana account encoding options
