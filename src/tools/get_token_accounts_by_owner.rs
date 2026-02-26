use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use super::json_to_text;
use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_token_accounts_by_owner",
    description = "Get all SPL token accounts owned by a wallet address.

Use this tool when you need to:
- View all token holdings for a wallet
- Check balance of a specific token (filter by mint)
- Find token accounts for Token or Token-2022 program

You MUST specify either mint OR program_id (not both).
Common program_ids:
- TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (Token Program)
- TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (Token-2022)"
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetTokenAccountsByOwnerTool {
    /// Owner wallet address (base58 encoded)
    pub owner_address: String,
    /// Filter by token mint address (mutually exclusive with program_id)
    pub mint: Option<String>,
    /// Filter by token program ID (mutually exclusive with mint)
    pub program_id: Option<String>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
}

impl GetTokenAccountsByOwnerTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        // Must specify exactly one of mint or program_id
        match (&self.mint, &self.program_id) {
            (None, None) => {
                return Ok(CallToolResult::text_content(vec![TextContent::from(
                    "Error: Must specify either 'mint' or 'program_id' parameter",
                )]));
            }
            (Some(_), Some(_)) => {
                return Ok(CallToolResult::text_content(vec![TextContent::from(
                    "Error: Specify only one of 'mint' or 'program_id', not both",
                )]));
            }
            _ => {}
        }

        let accounts = client
            .get_token_accounts_by_owner(
                &self.owner_address,
                self.mint.as_deref(),
                self.program_id.as_deref(),
                self.commitment.as_deref(),
            )
            .await
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&accounts).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
