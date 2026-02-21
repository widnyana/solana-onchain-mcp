---
status: pending
priority: p2
issue_id: 025
tags: [agent-native, usability, code-review]
dependencies: []
date: 2026-02-21
---

# confirm_transaction Not Exposed as Tool

## Problem Statement

After `transfer_sol` or `transfer_token` returns a signature, agents have no way to poll for confirmation. The `confirm_transaction` method exists in the client but is not exposed as an MCP tool.

## Findings

**Location**: `src/rpc/client.rs:432-438`

```rust
pub async fn confirm_transaction(&self, sig: &Signature) -> Result<bool> {
    // Method exists but no tool exposes it
}
```

**Agent workflow gap**:
```
1. Agent: Execute transfer_token
2. Agent: Receive signature
3. Agent: ??? (no way to verify transaction confirmed)
4. Agent: Cannot report success/failure to user reliably
```

**Impact**: Agents cannot verify transaction finality, leading to incomplete workflows.

## Proposed Solutions

### Option 1: Add get_signature_status Tool (Recommended)
Expose confirmation status as a new tool.

```rust
#[mcp_tool(
    name = "get_signature_status",
    description = "Check if a transaction signature has been confirmed..."
)]
pub struct GetSignatureStatusTool {
    pub signature: String,
    pub commitment: Option<String>,
}
```

**Pros**: Explicit, composable
**Cons**: Requires additional call
**Effort**: Small
**Risk**: Low

### Option 2: Include Status in Transfer Response
Make transfers wait for confirmation and include status.

```json
{
  "signature": "...",
  "confirmed": true,
  "slot": 12345
}
```

**Pros**: One call for full result
**Cons**: Blocks until confirmed, may timeout
**Effort**: Medium
**Risk**: Medium

## Recommended Action

Implement Option 1 - add `get_signature_status` tool for composable workflow.

## Technical Details

- **Affected Files**: New file `src/tools/get_signature_status.rs`, `src/tools/mod.rs`
- **Components**: Transaction status, confirmation

## Acceptance Criteria

- [ ] Create get_signature_status tool
- [ ] Wire to existing confirm_transaction method
- [ ] Return structured status response
- [ ] Add to mod.rs and handler
- [ ] Test confirmation flow

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- Related: src/rpc/client.rs:410-416 (confirm_transaction)
