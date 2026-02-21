---
status: pending
priority: p3
issue_id: 019
tags: [refactor, dry, code-review]
dependencies: []
date: 2026-02-21
---

# DRY: Add Tool Result Helper Functions

## Problem Statement

Every tool has nearly identical result wrapping code. The `CallToolResult::text_content(vec![TextContent::from(...)])` pattern is duplicated 8+ times.

## Findings

**Location**: All tool files in `src/tools/`

Duplicated pattern:
```rust
Ok(CallToolResult::text_content(vec![TextContent::from(
    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Fallback".to_string()),
)]))
```

**Impact**: ~40 lines of duplicated code across tool files.

## Proposed Solutions

### Option 1: Add Helper Functions (Recommended)

```rust
// In src/tools/mod.rs or new src/utils.rs
pub fn json_result<T: serde::Serialize>(value: T, fallback: &str) -> CallToolResult {
    CallToolResult::text_content(vec![TextContent::from(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| fallback.to_string()),
    )])
}

pub fn text_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::text_content(vec![TextContent::from(msg.into())])
}
```

**Usage:**
```rust
// Before
Ok(CallToolResult::text_content(vec![TextContent::from(
    "Error: At least one address is required",
)]))

// After
Ok(text_result("Error: At least one address is required"))
```

**Pros**: Clean syntax, DRY, consistent
**Cons**: Import needed
**Effort**: Small
**Risk**: None

## Recommended Action

Add helpers to `src/tools/mod.rs` and update all tool files.

## Technical Details

- **Affected Files**: `src/tools/mod.rs`, all tool files
- **LOC reduction**: ~40 lines

## Acceptance Criteria

- [ ] Add `json_result` helper
- [ ] Add `text_result` helper
- [ ] Update all tool files to use helpers
- [ ] Run `cargo check` - no errors
- [ ] Run tests - all pass

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Code simplicity review |

## Resources

- MCP SDK docs
