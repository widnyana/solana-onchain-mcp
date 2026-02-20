use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_slot",
    description = "Get the current slot (block height) from the Solana cluster."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetSlotTool {
    /// Commitment level (processed, confirmed, finalized)
    pub commitment: Option<String>,
}

impl GetSlotTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let slot = client
            .get_slot(self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        let message = format!("Current slot: {}", slot);

        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}
