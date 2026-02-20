use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_balance",
    description = "Get SOL balance for a Solana address. Returns balance in lamports (1 SOL = 1,000,000,000 lamports)."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetBalanceTool {
    /// The Solana address to check balance for
    pub address: String,
    /// Commitment level (processed, confirmed, finalized)
    pub commitment: Option<String>,
}

impl GetBalanceTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let balance = client
            .get_balance(&self.address, self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        let message = format!(
            "Balance: {} lamports ({:.9} SOL)",
            balance,
            balance as f64 / 1_000_000_000.0
        );

        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
