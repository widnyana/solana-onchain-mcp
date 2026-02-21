---
status: pending
priority: p3
issue_id: 018
tags: [refactor, dry, code-review]
dependencies: []
date: 2026-02-21
---

# DRY: Add ParsePubkeyExt Trait for Address Parsing

## Problem Statement

The pubkey parsing pattern with error mapping is repeated 7+ times across `client.rs`. Same boilerplate code with identical error handling.

## Findings

**Location**: `src/rpc/client.rs` (lines 83, 107, 136, 174, 219, 255, 292)

Repeated pattern:
```rust
.parse::<Pubkey>()
.map_err(|e| SolanaMcpError::InvalidAddress(e.to_string()))?;
```

**Impact**: ~14 lines of duplicated code, inconsistent error messages possible.

## Proposed Solutions

### Option 1: Add Extension Trait (Recommended)

```rust
// In a new utils module or error.rs
pub trait ParsePubkeyExt {
    fn parse_pubkey(&self) -> Result<Pubkey>;
}

impl ParsePubkeyExt for str {
    fn parse_pubkey(&self) -> Result<Pubkey> {
        self.parse::<Pubkey>()
            .map_err(|e| SolanaMcpError::InvalidAddress(e.to_string()))
    }
}

// Usage becomes:
let pubkey = address.parse_pubkey()?;
```

**Pros**: Clean syntax, consistent errors, DRY
**Cons**: External trait import needed
**Effort**: Small
**Risk**: None

### Option 2: Create Helper Function

```rust
fn parse_pubkey(s: &str) -> Result<Pubkey> {
    s.parse::<Pubkey>()
        .map_err(|e| SolanaMcpError::InvalidAddress(e.to_string()))
}
```

**Pros**: Simpler, no trait
**Cons**: More verbose call site
**Effort**: Small
**Risk**: None

## Recommended Action

Implement Option 1 for cleaner syntax. Add trait to a new `src/utils.rs` module.

## Technical Details

- **Affected Files**: `src/rpc/client.rs`, new `src/utils.rs`
- **LOC reduction**: ~14 lines
- **Import needed**: `use crate::utils::ParsePubkeyExt;`

## Acceptance Criteria

- [ ] Create `src/utils.rs` with trait
- [ ] Add `ParsePubkeyExt` trait implementation
- [ ] Update all 7+ call sites in client.rs
- [ ] Run `cargo check` - no errors
- [ ] Run tests - all pass

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Code simplicity review |

## Resources

- Rust extension trait pattern
