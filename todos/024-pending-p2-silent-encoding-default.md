---
status: pending
priority: p2
issue_id: 024
tags: [architecture, validation, ux, code-review]
dependencies: []
date: 2026-02-21
---

# Silent Default for Invalid Encoding Values

## Problem Statement

The `parse_encoding` function silently defaults to `base64` for invalid encoding values, with only a tracing warning that agents cannot see.

## Findings

**Location**: `src/rpc/client.rs:55-64`

```rust
fn parse_encoding(encoding: &str) -> UiAccountEncoding {
    match encoding {
        "base58" => UiAccountEncoding::Base58,
        "jsonParsed" => UiAccountEncoding::JsonParsed,
        _ => {
            tracing::warn!("Invalid encoding '{}', defaulting to base64", encoding);
            UiAccountEncoding::Base64
        }
    }
}
```

**Problems**:
1. Agent passes "base59" typo, gets results but with wrong encoding
2. Harder to debug than immediate error
3. Inconsistent with address parsing which returns `InvalidAddress` error
4. Tracing warnings not visible to agents

**Impact**: Users may receive unexpected data format, agents have incorrect mental model of system behavior.

## Proposed Solutions

### Option 1: Return Result with Error (Recommended)
Make encoding parsing fallible.

```rust
fn parse_encoding(encoding: &str) -> Result<UiAccountEncoding, SolanaMcpError> {
    match encoding {
        "base58" => Ok(UiAccountEncoding::Base58),
        "base64" => Ok(UiAccountEncoding::Base64),
        "jsonParsed" => Ok(UiAccountEncoding::JsonParsed),
        other => Err(SolanaMcpError::InvalidEncoding(other.to_string())),
    }
}
```

**Pros**: Fail-fast, consistent with other parsing
**Cons**: Breaking change
**Effort**: Medium
**Risk**: Low

### Option 2: Add Warning to Response
Keep silent default but include warning in response.

```json
{
  "warning": "Encoding 'invalid' not recognized, defaulted to base64",
  "valid_options": ["base64", "base58", "jsonParsed"]
}
```

**Pros**: Non-breaking, visible to agent
**Cons**: Still silent correction
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 1 - return Result. Fail-fast is the correct pattern for invalid input.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, potentially tool files
- **Components**: Input validation

## Acceptance Criteria

- [ ] Change parse_encoding to return Result
- [ ] Add InvalidEncoding variant to SolanaMcpError
- [ ] Update callers to handle error
- [ ] Add test for invalid encoding

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Related: parse_commitment (same pattern issue)
