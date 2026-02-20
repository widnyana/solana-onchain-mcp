use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "simulate_transaction",
    description = "Simulate a transaction without signing or submitting it.

Use this tool when you need to:
- Test transaction behavior before signing
- Check compute units and fees
- Debug transaction failures
- Verify instruction logic

Provide a base64-encoded serialized transaction.
Use replace_recent_blockhash=true to use current blockhash for testing.

Returns logs, compute units consumed, and any errors."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct SimulateTransactionTool {
    /// Base64 encoded serialized transaction
    pub transaction: String,
    /// Encoding of the transaction: "base64" (default) | "base58"
    pub encoding: Option<String>,
    /// Replace transaction blockhash with recent one (useful for testing old transactions)
    pub replace_recent_blockhash: Option<bool>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
}

impl SimulateTransactionTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let result = client
            .simulate_transaction(
                &self.transaction,
                self.encoding.as_deref(),
                self.replace_recent_blockhash.unwrap_or(false),
                self.commitment.as_deref(),
            )
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Simulation result".to_string()),
        )]))
    }
}
