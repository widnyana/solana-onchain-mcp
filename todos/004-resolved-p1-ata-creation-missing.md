---
status: resolved
priority: p1
issue_id: 004
tags: [agent-native, usability, token, code-review]
dependencies: []
date: 2026-02-21
---

# transfer_token: No ATA Creation/Check Capability

## Problem Statement

Token transfers fail if the recipient doesn't have an Associated Token Account (ATA). The tool description mentions this requirement but provides no way to create or check for ATAs.

## Findings

**Location**: `src/tools/transfer_token.rs:12-14`

- Description says "Recipient must have an associated token account (ATA)"
- No tool to check if ATA exists
- No tool to create ATA if missing
- Transfer will fail silently without ATA

```rust
// Current documentation says:
// "Recipient must have an associated token account (ATA)"
// But provides no way to ensure this
```

## Proposed Solutions

### Option 1: Auto-Create ATA (Recommended for UX)
Modify transfer_token to auto-create ATA when missing.

```rust
// Check if ATA exists, create if not
let ata = get_associated_token_address(&recipient, &mint);
if rpc.get_account(&ata)?.is_none() {
    create_associated_token_account(...)?;
}
```

**Pros**: Seamless user/agent experience
**Cons**: More complex, additional transaction
**Effort**: Medium
**Risk**: Low

### Option 2: Add Separate ATA Tools
Create dedicated tools for ATA operations.

```rust
// get_associated_token_address(owner, mint) -> address
// create_associated_token_account(owner, mint) -> signature
```

**Pros**: Explicit control, composable
**Cons**: Requires multiple calls
**Effort**: Medium
**Risk**: Low

### Option 3: Document Workaround (Quick Fix)
Document that agents should check ATA existence via get_token_accounts_by_owner first.

**Pros**: Immediate guidance
**Cons**: Doesn't solve the problem
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 2 (Separate ATA Tools) - provides explicit control and composability.

## Technical Details

- **Affected Files**: `src/tools/transfer_token.rs`, new file needed
- **Components**: Token operations
- **Agent Impact**: HIGH - transfers may fail unexpectedly

## Acceptance Criteria

- [ ] Create `get_associated_token_address` tool
- [ ] Create `create_associated_token_account` tool
- [ ] Update transfer_token description with ATA guidance
- [ ] Add test for ATA creation flow
- [ ] Document common token transfer workflow

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- SPL Associated Token Account program docs
- Related: `src/tools/transfer_token.rs`
