---
status: pending
priority: p3
issue_id: 013
tags: [code-quality, maintainability, code-review]
dependencies: []
date: 2026-02-21
---

# Magic Numbers Without Named Constants

## Problem Statement

Several numeric limits and defaults are hardcoded without named constants, making them harder to find and modify.

## Findings

**Locations**:
- `src/tools/get_multiple_accounts.rs:37`: `100` (max accounts)
- `src/tools/get_signatures_for_address.rs:36`: `100` (default), `1000` (max)

```rust
if self.addresses.len() > 100 { ... }
let limit = self.limit.unwrap_or(100).clamp(1, 1000);
```

## Proposed Solutions

### Option 1: Define Constants (Recommended)
Create named constants for all magic numbers.

```rust
const MAX_ACCOUNTS_PER_REQUEST: usize = 100;
const MAX_SIGNATURES_LIMIT: usize = 1000;
const DEFAULT_SIGNATURES_LIMIT: usize = 100;
```

**Pros**: Self-documenting, easy to modify
**Cons**: Minor boilerplate
**Effort**: Small
**Risk**: None

## Recommended Action

Define constants at module level and replace magic numbers.

## Technical Details

- **Affected Files**: `src/tools/get_multiple_accounts.rs`, `src/tools/get_signatures_for_address.rs`
- **Components**: Input validation

## Acceptance Criteria

- [ ] Define named constants
- [ ] Replace all magic numbers
- [ ] Verify behavior unchanged

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Architecture review |

## Resources

- Related: P1-001 (unbounded memory)
