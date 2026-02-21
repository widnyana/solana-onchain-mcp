use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_signatures_for_address",
    description = "Get transaction signatures for an address, ordered newest to oldest.

Use this tool when you need to:
- View transaction history for a wallet or account
- Paginate through historical transactions
- Check transaction status (success/failure)

Returns signatures with slot, blockTime, err (null if success), and confirmationStatus.
Use 'before' and 'until' for pagination."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetSignaturesForAddressTool {
    /// The Solana address to get transaction signatures for
    pub address: String,
    /// Maximum number of signatures to return (1-1000, default 100)
    pub limit: Option<u64>,
    /// Pagination: start searching backward from this signature
    pub before: Option<String>,
    /// Pagination: search until this signature
    pub until: Option<String>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
}

impl GetSignaturesForAddressTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let limit = self.limit.unwrap_or(100).clamp(1, 1000);

        let signatures = client
            .get_signatures_for_address(
                &self.address,
                Some(limit as usize),
                self.before.as_deref(),
                self.until.as_deref(),
                self.commitment.as_deref(),
            )
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&signatures).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
