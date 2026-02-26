use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};

use super::json_to_text;
use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_transaction",
    description = "Fetch transaction details by signature. Returns full transaction data \
including status, fees, and account changes. Use this when the user asks about a \
specific transaction, its status, or what it did."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetTransactionTool {
    /// The transaction signature to fetch
    pub signature: String,
    /// Commitment level: "processed" (fastest, may rollback), \
    /// "confirmed" (default, ~400ms latency), "finalized" (~1s, permanent)
    pub commitment: Option<String>,
}

impl GetTransactionTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let tx = client
            .get_transaction(&self.signature, self.commitment.as_deref())
            .await
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&tx).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
