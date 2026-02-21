---
status: pending
priority: p3
issue_id: 027
tags: [performance, memory, code-review]
dependencies: []
date: 2026-02-21
---

# Unnecessary Allocation in sanitize_rpc_message

## Problem Statement

The `sanitize_rpc_message` function allocates a lowercase copy of the message before pattern matching, even when patterns will match and the allocation is wasted.

## Findings

**Location**: `src/error.rs:45-71`

```rust
fn sanitize_rpc_message(msg: &str) -> String {
    let msg_lower = msg.to_lowercase();  // Allocation: ALWAYS happens
    for pattern in sensitive_patterns {
        if msg_lower.contains(pattern) {
            return "RPC request failed...".to_string();
            // msg_lower is now garbage - wasted allocation
        }
    }
    // ...
}
```

**Impact**: For every RPC error with a sensitive pattern match, we allocate and immediately discard `msg.len()` bytes.

## Proposed Solutions

### Option 1: Case-Insensitive Matching Without Allocation (Recommended)
Use ASCII case-insensitive comparison.

```rust
fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    haystack.as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle.as_bytes()))
}
```

**Pros**: Eliminates allocation on error path
**Cons**: Slightly more complex code
**Effort**: Small
**Risk**: Low

### Option 2: Accept Current Implementation
The allocation is on the error path (cold code) and may not be worth optimizing.

**Pros**: No change needed
**Cons**: Wasted allocation
**Effort**: None
**Risk**: None

## Recommended Action

Consider Option 1 if performance is critical, otherwise accept current implementation (Option 2).

## Technical Details

- **Affected Files**: `src/error.rs`
- **Components**: Error handling, memory

## Acceptance Criteria

- [ ] Decide on optimization approach
- [ ] If optimizing, implement case-insensitive matching
- [ ] Add test for new function

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Performance review |

## Resources

- Rust case-insensitive string matching patterns
