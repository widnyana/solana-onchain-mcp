---
status: pending
priority: p3
issue_id: 028
tags: [architecture, code-review]
dependencies: []
date: 2026-02-21
---

# json_to_text Placement in tools/mod.rs

## Problem Statement

The `json_to_text` utility function is placed in `tools/mod.rs` which serves as a tool registry. Mixing orchestration with utility functions creates unclear module responsibility.

## Findings

**Location**: `src/tools/mod.rs:43-46`

`mod.rs` currently contains:
- Module declarations
- Re-exports (`pub use`)
- `tool_box!` macro for tool registration
- `json_to_text` utility function

**Existing Pattern Analysis**:
- `config.rs` has module-level helpers (`is_private_ip()`, `validate_custom_url()`)
- `error.rs` has module-level helper (`sanitize_rpc_message()`)

## Proposed Solutions

### Option 1: Accept Current Placement
The function is used by tools, so placement in tools module is defensible.

**Pros**: No change needed, function is close to usage
**Cons**: Module has mixed responsibilities
**Effort**: None
**Risk**: None

### Option 2: Create Dedicated Utils Module
Move to `src/utils.rs` or similar.

```rust
// src/utils.rs
pub fn json_to_text<T: serde::Serialize>(value: &T) -> TextContent { ... }
```

**Pros**: Clear module responsibility
**Cons**: Additional file, import changes
**Effort**: Small
**Risk**: Low

## Recommended Action

Accept Option 1 - current placement is acceptable. The function is only used by tools and keeping it in the tools module is reasonable.

## Technical Details

- **Affected Files**: `src/tools/mod.rs` (or new `src/utils.rs`)
- **Components**: Code organization

## Acceptance Criteria

- [ ] Decide on placement
- [ ] If moving, update imports in all tool files

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Rust module organization patterns
