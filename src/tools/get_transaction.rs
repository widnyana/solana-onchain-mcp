use rust_mcp_sdk::macros::{mcp_tool, JsonSchema};
use rust_mcp_sdk::schema::{CallToolResult, CallToolError, TextContent};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_transaction",
    description = "Get transaction details by signature. Returns parsed transaction data in JSON format."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetTransactionTool {
    /// The transaction signature to fetch
    pub signature: String,
    /// Commitment level (processed, confirmed, finalized)
    pub commitment: Option<String>,
}

impl GetTransactionTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let tx = client
            .get_transaction(&self.signature, self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        let message = serde_json::to_string_pretty(&tx)
            .unwrap_or_else(|_| "Failed to serialize transaction".to_string());

        Ok(CallToolResult::text_content(vec![TextContent::from(message)]))
    }
}
