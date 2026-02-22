//! Raw Transaction Inspector Tool
//!
//! Provides detailed raw transaction data with instruction decoding.

use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

use super::{format_instruction_error, get_program_name};
use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "inspect_transaction_raw",
    description = "Get detailed raw transaction data with instruction decoding. Returns full transaction structure including accounts, instructions with decoded data, and program identification. Use for debugging transaction structure."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InspectTransactionRawTool {
    /// Transaction signature (base58 encoded)
    pub signature: String,
    /// Commitment level: processed, confirmed, or finalized (default: confirmed)
    pub commitment: Option<String>,
}

impl InspectTransactionRawTool {
    pub fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        // Use the existing get_transaction method which returns JSON
        let tx = client
            .get_transaction(&self.signature, self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        // Add program name annotations
        let annotated = annotate_with_program_names(tx);

        let message =
            serde_json::to_string_pretty(&annotated).unwrap_or_else(|_| "Failed to serialize transaction".to_string());

        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

/// Add program name annotations to transaction JSON
fn annotate_with_program_names(mut tx: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = tx.as_object_mut() {
        // Add program names to the transaction object
        if let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut())
            && let Some(err) = meta.get("err")
            && !err.is_null()
        {
            let error_interpretation = format_instruction_error(err);
            meta.insert(
                "error_interpretation".to_string(),
                serde_json::json!(error_interpretation),
            );
        }

        // Add program info to instructions
        if let Some(tx_obj) = obj.get_mut("transaction").and_then(|t| t.as_object_mut())
            && let Some(message) = tx_obj.get_mut("message").and_then(|m| m.as_object_mut())
        {
            // First, extract account keys to a Vec<String>
            let account_keys: Vec<String> = message
                .get("accountKeys")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|k| {
                            k.as_object()
                                .and_then(|obj| obj.get("pubkey"))
                                .or(Some(k))
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Now mutably borrow to add program names
            if let Some(instructions) = message.get_mut("instructions").and_then(|i| i.as_array_mut()) {
                for instr in instructions.iter_mut() {
                    if let Some(instr_obj) = instr.as_object_mut()
                        && let Some(program_id_index) = instr_obj.get("programIdIndex").and_then(|p| p.as_u64())
                        && let Some(program_id) = account_keys.get(program_id_index as usize)
                        && let Some(program_name) = get_program_name(program_id)
                    {
                        instr_obj.insert("programName".to_string(), serde_json::json!(program_name));
                    }
                }
            }
        }
    }
    tx
}
