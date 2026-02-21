use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_multiple_accounts",
    description = "Batch fetch multiple Solana accounts in a single request (up to 100).

Use this tool when you need to:
- Fetch multiple accounts efficiently in one call
- Compare data across several accounts
- Reduce RPC calls when checking multiple addresses

Returns an array where null indicates non-existent accounts."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetMultipleAccountsTool {
    /// Array of Solana addresses to query (max 100, base58 encoded)
    pub addresses: Vec<String>,
    /// Encoding: "base64" (default) | "base58" | "jsonParsed"
    pub encoding: Option<String>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
}

impl GetMultipleAccountsTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        if self.addresses.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                "Error: At least one address is required",
            )]));
        }

        if self.addresses.len() > 100 {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                "Error: Maximum 100 addresses allowed per request",
            )]));
        }

        let accounts = client
            .get_multiple_accounts(
                &self.addresses,
                self.encoding.as_deref(),
                self.commitment.as_deref(),
            )
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&accounts).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
