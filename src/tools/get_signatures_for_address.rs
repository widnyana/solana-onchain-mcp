use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};

use super::json_to_text;
use crate::rpc::SolanaRpcClient;

/// Default number of signatures to return when limit is not specified.
const DEFAULT_SIGNATURES_LIMIT: u64 = 100;
/// Maximum number of signatures that can be returned in a single request.
const MAX_SIGNATURES_LIMIT: u64 = 1000;

#[mcp_tool(
    name = "get_signatures_for_address",
    description = "Get transaction SIGNATURES (not full details) for an address, ordered newest to oldest.

Use this tool ONLY when you need:
- A list of signatures for pagination purposes
- To check if signatures exist (without fetching details)
- Transaction status checking **without** full transaction data

Returns ONLY signatures with slot, blockTime, err (null if success), and confirmationStatus.
Does NOT include transaction details, instructions, or human-readable explanations.

For full transaction details with classification, use `query_transactions` instead.

Pagination: Use 'before' and 'until' parameters.

**NEVER** truncate the `signature` field"
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
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
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let limit = self
            .limit
            .unwrap_or(DEFAULT_SIGNATURES_LIMIT)
            .clamp(1, MAX_SIGNATURES_LIMIT);

        let signatures = client
            .get_signatures_for_address(
                &self.address,
                Some(limit as usize),
                self.before.as_deref(),
                self.until.as_deref(),
                self.commitment.as_deref(),
            )
            .await
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&signatures).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
