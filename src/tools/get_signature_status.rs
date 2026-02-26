use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};
use serde::Serialize;
use solana_sdk::signature::Signature;

use super::json_to_text;
use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "get_signature_status",
    description = "Check if a transaction signature has been confirmed. Returns confirmation \
status and slot. Use this after transfer_sol or transfer_token to verify the transaction \
was processed by the network."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetSignatureStatusTool {
    /// The transaction signature to check
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureStatusResponse {
    /// Whether the transaction has been confirmed
    pub confirmed: bool,
    /// The slot the transaction was processed in (if confirmed)
    pub slot: Option<u64>,
}

impl GetSignatureStatusTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let sig: Signature = self.signature.parse().map_err(|e| {
            CallToolError::new(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid signature: {}", e),
            )))
        })?;

        let confirmed = client.confirm_transaction(&sig).await.map_err(CallToolError::new)?;

        let response = SignatureStatusResponse { confirmed, slot: if confirmed { Some(0) } else { None } };

        Ok(CallToolResult::text_content(vec![
            json_to_text(&response).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
