---
status: pending
priority: p2
issue_id: 009
tags: [agent-native, usability, code-review]
dependencies: []
date: 2026-02-21
---

# Missing Transaction Confirmation Status Tool

## Problem Statement

`transfer_sol` and `transfer_token` return a signature but there is no tool to check if the transaction was confirmed. Agents cannot verify transfer success.

## Findings

**Location**: Handler has `confirm_transaction` method but no tool exposes it

- Transfers return signature only
- No way to check confirmation status
- No polling mechanism for finality

## Proposed Solutions

### Option 1: Add get_signature_status Tool (Recommended)
Expose confirmation status as a new tool.

```rust
#[mcp_tool(name = "get_signature_status")]
pub struct GetSignatureStatusTool {
    pub signature: String,
    pub commitment: Option<String>,
}

// Returns: { confirmed: bool, slot: N, err: null|string }
```

**Pros**: Explicit, composable
**Cons**: Requires additional call
**Effort**: Small
**Risk**: Low

### Option 2: Include Status in Transfer Response
Make transfers wait for confirmation and include status.

```rust
// Transfer returns: { signature, confirmed: bool, slot: N }
```

**Pros**: One call for full result
**Cons**: Blocks until confirmed, may timeout
**Effort**: Medium
**Risk**: Medium

## Recommended Action

Implement Option 1 - add `get_signature_status` tool for composable workflow.

## Technical Details

- **Affected Files**: New file `src/tools/get_signature_status.rs`
- **Components**: Transaction status, confirmation

## Acceptance Criteria

- [ ] Create `get_signature_status` tool
- [ ] Wire to existing `confirm_transaction` method
- [ ] Return structured status response
- [ ] Add to mod.rs and handler
- [ ] Test confirmation flow

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- Related: `src/rpc/client.rs:410-416` (confirm_transaction)
