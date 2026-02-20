use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_account_info",
    description = "Get account information including owner, lamports, data size, and executable status.

Use this tool when you need to:
- Inspect any Solana account's data and metadata
- Check if an account exists (returns null if not)
- Get the owner program of an account
- View account balance in lamports

Note: For token balances, use get_token_accounts_by_owner instead."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetAccountInfoTool {
    /// The Solana address to query (base58 encoded)
    pub address: String,
    /// Encoding: "base64" (default) | "base58" | "jsonParsed"
    pub encoding: Option<String>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
}

impl GetAccountInfoTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let account = client
            .get_account_info(
                &self.address,
                self.encoding.as_deref(),
                self.commitment.as_deref(),
            )
            .map_err(CallToolError::new)?;

        match account {
            Some(acc) => Ok(CallToolResult::text_content(vec![TextContent::from(
                serde_json::to_string_pretty(&acc).unwrap_or_else(|_| "Account data".to_string()),
            )])),
            None => Ok(CallToolResult::text_content(vec![TextContent::from(
                "Account not found or does not exist",
            )])),
        }
    }
}
