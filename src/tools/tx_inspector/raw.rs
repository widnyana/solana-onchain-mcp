//! Raw Transaction Inspector Tool
//!
//! Provides detailed raw transaction data with instruction decoding.

use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

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
        if let Some(meta) = obj.get_mut("meta").and_then(|m| m.as_object_mut()) {
            // Add error interpretation
            if let Some(err) = meta.get("err") {
                if !err.is_null() {
                    let error_interpretation = interpret_error(err);
                    meta.insert(
                        "error_interpretation".to_string(),
                        serde_json::json!(error_interpretation),
                    );
                }
            }
        }

        // Add program info to instructions
        if let Some(tx_obj) = obj.get_mut("transaction").and_then(|t| t.as_object_mut()) {
            if let Some(message) = tx_obj.get_mut("message").and_then(|m| m.as_object_mut()) {
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
                        if let Some(instr_obj) = instr.as_object_mut() {
                            if let Some(program_id_index) = instr_obj.get("programIdIndex").and_then(|p| p.as_u64()) {
                                if let Some(program_id) = account_keys.get(program_id_index as usize) {
                                    if let Some(program_name) = get_program_name(program_id) {
                                        instr_obj.insert("programName".to_string(), serde_json::json!(program_name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    tx
}

fn get_program_name(pubkey: &str) -> Option<&'static str> {
    let programs = [
        ("11111111111111111111111111111111", "System Program"),
        (
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "Token Program",
        ),
        (
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
            "Token2022 Program",
        ),
        (
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
            "Associated Token Account",
        ),
        (
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            "Raydium DEX",
        ),
        (
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "Jupiter Aggregator",
        ),
        ("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctWtC", "Orca DEX"),
    ];

    for (id, name) in programs {
        if id == pubkey {
            return Some(name);
        }
    }
    None
}

fn interpret_error(err: &serde_json::Value) -> String {
    if let Some(obj) = err.as_object() {
        if let Some(instr_err) = obj.get("InstructionError") {
            if let Some(arr) = instr_err.as_array() {
                if arr.len() == 2 {
                    let index = &arr[0];
                    let error_type = &arr[1];
                    return format!(
                        "Instruction {} failed: {}",
                        index,
                        serde_json::to_string(error_type).unwrap_or_else(|_| "unknown error".to_string())
                    );
                }
            }
        }
    }
    serde_json::to_string(err).unwrap_or_else(|_| "unknown error".to_string())
}
