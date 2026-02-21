use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};

use super::json_to_text;
use crate::rpc::SolanaRpcClient;

/// Filter for memcmp comparison at a specific offset
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct MemcmpFilter {
    /// Offset into account data to start comparison
    pub offset: u64,
    /// Bytes to compare (base58 encoded)
    pub bytes: String,
}

#[mcp_tool(
    name = "get_program_accounts",
    description = "Get all accounts owned by a program.

Use this tool when you need to:
- Query all accounts for a dApp/program
- Filter accounts by data size or content
- Explore program state

REQUIRED: At least one filter (data_size or memcmp) is mandatory to prevent resource exhaustion.
This ensures the query is scoped before making the RPC call.

Common program IDs:
- TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (Token Program)
- TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb (Token-2022)"
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetProgramAccountsTool {
    /// Program ID to query accounts for (base58 encoded)
    pub program_id: String,
    /// Filter by account data size (optional)
    pub data_size: Option<u64>,
    /// Filter by memcmp at offset (optional)
    pub memcmp: Option<MemcmpFilter>,
    /// Commitment level: "processed" | "confirmed" (default) | "finalized"
    pub commitment: Option<String>,
    /// Encoding: "base64" (default) | "base58" | "jsonParsed"
    pub encoding: Option<String>,
}

impl GetProgramAccountsTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        let accounts = client
            .get_program_accounts(
                &self.program_id,
                self.data_size.map(|s| s as usize),
                self.memcmp.as_ref().map(|m| (m.offset as usize, m.bytes.as_str())),
                self.commitment.as_deref(),
                self.encoding.as_deref(),
            )
            .map_err(CallToolError::new)?;

        Ok(CallToolResult::text_content(vec![
            json_to_text(&accounts).map_err(|e| CallToolError::new(Box::new(e)))?,
        ]))
    }
}
