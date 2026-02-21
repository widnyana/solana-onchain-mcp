---
status: pending
priority: p1
issue_id: 021
tags: [reliability, error-handling, code-review]
dependencies: []
date: 2026-02-21
---

# json_to_text Uses expect() - Panic Risk

## Problem Statement

The `json_to_text` helper function uses `.expect()` which will panic the entire server process if serialization fails, rather than returning an error.

## Findings

**Location**: `src/tools/mod.rs:45-46`

```rust
pub fn json_to_text<T: serde::Serialize>(value: &T) -> TextContent {
    TextContent::from(serde_json::to_string(value).expect("RPC response serialization cannot fail"))
}
```

**Problems**:
1. Serialization CAN fail for types with custom `Serialize` implementations that error
2. Deeply nested structures can exceed recursion limits
3. Values with invalid UTF-8 in string fields can fail
4. A panic crashes the entire MCP server, not just the request

**Impact**: Denial of service potential if a malicious or malformed RPC response triggers serialization failure.

## Proposed Solutions

### Option 1: Return Result (Recommended)
Change signature to return `Result` and let callers handle errors.

```rust
pub fn json_to_text<T: serde::Serialize>(value: &T) -> Result<TextContent, SolanaMcpError> {
    serde_json::to_string(value)
        .map(TextContent::from)
        .map_err(SolanaMcpError::SerializationError)
}
```

**Pros**: Proper error propagation, no hidden panics
**Cons**: All callers need to handle Result
**Effort**: Medium
**Risk**: Low

### Option 2: Use unwrap_or_default
Fall back to empty or error JSON on failure.

```rust
pub fn json_to_text<T: serde::Serialize>(value: &T) -> TextContent {
    TextContent::from(
        serde_json::to_string(value).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string())
    )
}
```

**Pros**: No signature change
**Cons**: Silent failure, loses data
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1 - return Result. This is the correct Rust pattern and allows proper error handling.

## Technical Details

- **Affected Files**: `src/tools/mod.rs`, all tool files that call `json_to_text`
- **Components**: Error handling, serialization

## Acceptance Criteria

- [ ] Change json_to_text to return Result
- [ ] Update all callers to handle Result
- [ ] Add test for serialization failure case
- [ ] Verify no panics on malformed data

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Security + Simplicity review |

## Resources

- Rust error handling best practices
