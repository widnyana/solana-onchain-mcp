---
status: pending
priority: p1
issue_id: 022
tags: [agent-native, usability, workflow, code-review]
dependencies: []
date: 2026-02-21
---

# Missing Tool: Create Associated Token Account (ATA)

## Problem Statement

The `transfer_token` tool requires the recipient to have an ATA, but there is no tool to create one. Agents cannot complete the transfer workflow when the recipient lacks an ATA.

## Findings

**Location**: `src/tools/transfer_token.rs:15-19` (description mentions requirement)

The description correctly warns:
> "IMPORTANT: The recipient MUST have an Associated Token Account (ATA) for the token mint."

And suggests checking with `get_token_accounts_by_owner`. However, if the check reveals no ATA exists, **there is no tool to create one**. The agent is stuck.

**Agent workflow failure**:
```
1. User: "Send 10 USDC to recipient X"
2. Agent: Checks recipient's token accounts (tool exists)
3. Agent: Finds no ATA for USDC
4. Agent: ??? (no tool to create ATA)
5. FAIL: Cannot complete request
```

**Impact**: Transfer token tool is incomplete - agents can verify prerequisites but cannot satisfy them.

## Proposed Solutions

### Option 1: Add create_ata Tool (Recommended)
Create a new tool that creates an ATA for a given owner and mint.

```rust
#[mcp_tool(
    name = "create_associated_token_account",
    description = "Create an Associated Token Account (ATA) for a token mint..."
)]
pub struct CreateAtaTool {
    pub owner: String,
    pub token_mint: String,
}
```

**Pros**: Completes the workflow, explicit action
**Cons**: Additional complexity, requires rent-exempt SOL
**Effort**: Medium
**Risk**: Low

### Option 2: Auto-create ATA in transfer_token
Add a parameter to automatically create ATA if missing.

```rust
pub struct TransferTokenTool {
    // ...
    pub create_ata_if_missing: Option<bool>,
}
```

**Pros**: Simpler for agents
**Cons**: Hidden behavior, may surprise users
**Effort**: Medium
**Risk**: Medium

## Recommended Action

Implement Option 1 - add `create_ata` tool. This is explicit and gives agents full control over the workflow.

## Technical Details

- **Affected Files**: New file `src/tools/create_ata.rs`, `src/tools/mod.rs`, `src/handler.rs`
- **Components**: SPL Associated Token Account program

## Acceptance Criteria

- [ ] Create create_ata tool
- [ ] Register in tool_box!
- [ ] Wire into handler
- [ ] Update transfer_token description to reference new tool
- [ ] Add tests

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- SPL Associated Token Account documentation
- Related: transfer_token.rs
