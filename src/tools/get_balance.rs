use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_balance",
    description = "Get SOL balance for a Solana address. Returns balance in lamports \
(1 SOL = 1,000,000,000 lamports). Use this when the user asks about wallet balance, \
account funds, or SOL holdings. Does NOT return SPL token balances - those require \
a different RPC call.

**DO NOT** add commentary."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetBalanceTool {
    /// The Solana address to check balance for
    pub address: String,
    /// Commitment level: "processed" (fastest, may rollback), \
    /// "confirmed" (default, ~400ms latency), "finalized" (~1s, permanent)
    pub commitment: Option<String>,
}

impl GetBalanceTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let balance = client
            .get_balance(&self.address, self.commitment.as_deref())
            .await
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
