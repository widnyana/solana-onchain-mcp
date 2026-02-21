---
status: resolved
priority: p1
issue_id: 003
tags: [agent-native, usability, code-review]
dependencies: []
date: 2026-02-21
---

# simulate_transaction: No Way to Build Transactions

## Problem Statement

The `simulate_transaction` tool accepts a serialized transaction but provides NO way for agents to build/serialize transactions. This makes the tool unusable for AI agents without external tooling.

## Findings

**Location**: `src/tools/simulate_transaction.rs:9-22`

- Tool requires base64-encoded serialized transaction
- No tool exists to create transactions from instructions
- No tool exists to serialize transactions
- No tool exists to build common transaction types (transfer, token operations)

```rust
// Current: Agent receives this but cannot fulfill
pub transaction: String,  // Base64 encoded serialized transaction - HOW?
```

## Proposed Solutions

### Option 1: Add Transaction Building Tools
Create new tools for transaction construction.

```rust
// build_transfer_tx(from, to, amount) -> base64_tx
// build_token_transfer_tx(owner, mint, to, amount) -> base64_tx
// serialize_transaction(instructions, payer) -> base64_tx
```

**Pros**: Full flexibility for agents
**Cons**: Significant implementation effort
**Effort**: Large
**Risk**: Medium (security considerations)

### Option 2: Accept Higher-Level Parameters
Modify simulate_transaction to accept instruction-level parameters.

**Pros**: Simpler for agents
**Cons**: Limits flexibility, breaks current API
**Effort**: Medium
**Risk**: Medium

### Option 3: Document Current Limitation (Quick Fix)
Update tool description to clearly state this is for advanced users with pre-built transactions.

**Pros**: Immediate clarity
**Cons**: Doesn't solve the problem
**Effort**: Small
**Risk**: Low

## Recommended Action

Implement Option 3 immediately (documentation update). Plan Option 1 for next iteration.

## Technical Details

- **Affected Files**: `src/tools/simulate_transaction.rs`
- **Components**: MCP tool descriptions
- **Agent Impact**: HIGH - tool currently unusable by agents

## Acceptance Criteria

- [ ] Update tool description to clarify pre-built transaction requirement
- [ ] Add example of how to use with external serialization
- [ ] Document as known limitation
- [ ] Create feature request for transaction building tools

## Work Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-02-21 | Issue identified | Agent-native review |

## Resources

- Solana transaction serialization docs
- Related: transfer_sol.rs, transfer_token.rs (for reference implementation)
