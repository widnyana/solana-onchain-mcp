use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_transaction",
    description = "Fetch transaction details by signature. Returns full transaction data \
including status, fees, and account changes. Use this when the user asks about a \
specific transaction, its status, or what it did."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetTransactionTool {
    /// The transaction signature to fetch
    pub signature: String,
    /// Commitment level: "processed" (fastest, may rollback), \
    /// "confirmed" (default, ~400ms latency), "finalized" (~1s, permanent)
    pub commitment: Option<String>,
}

impl GetTransactionTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let tx = client
            .get_transaction(&self.signature, self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        let message =
            serde_json::to_string_pretty(&tx).unwrap_or_else(|_| "Failed to serialize transaction".to_string());

        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
