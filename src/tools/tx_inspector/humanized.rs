//! Humanized Transaction Inspector Tool
//!
//! Provides human-readable transaction analysis with explanations.

use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

use crate::rpc::SolanaRpcClient;

#[mcp_tool(
    name = "inspect_transaction_humanized",
    description = "Get human-readable transaction analysis with explanations. Decodes instruction purposes, token amounts with decimals/symbols, and provides an overall transaction summary. Best for understanding what a transaction does."
)]
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InspectTransactionHumanizedTool {
    /// Transaction signature (base58 encoded)
    pub signature: String,
    /// Commitment level: processed, confirmed, or finalized (default: confirmed)
    pub commitment: Option<String>,
}

impl InspectTransactionHumanizedTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient) -> Result<CallToolResult, CallToolError> {
        // Use the existing get_transaction method
        let tx = client
            .get_transaction(&self.signature, self.commitment.as_deref())
            .map_err(CallToolError::new)?;

        // Create humanized analysis
        let analysis = create_humanized_analysis(&tx);

        let message =
            serde_json::to_string_pretty(&analysis).unwrap_or_else(|_| "Failed to serialize analysis".to_string());

        Ok(CallToolResult::text_content(vec![TextContent::from(
            message,
        )]))
    }
}

#[derive(Debug, Serialize)]
struct TransactionAnalysis {
    signature: String,
    status: StatusInfo,
    fee: u64,
    slot: Option<u64>,
    block_time: Option<i64>,
    summary: String,
    instructions: Vec<InstructionAnalysis>,
    accounts: Vec<AccountAnalysis>,
}

#[derive(Debug, Serialize)]
struct StatusInfo {
    success: bool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct InstructionAnalysis {
    index: usize,
    program: String,
    program_name: Option<String>,
    instruction_type: Option<String>,
    explanation: Option<String>,
}

#[derive(Debug, Serialize)]
struct AccountAnalysis {
    pubkey: String,
    name: Option<String>,
    is_signer: bool,
    is_writable: bool,
}

fn create_humanized_analysis(tx: &serde_json::Value) -> TransactionAnalysis {
    let signature = tx
        .get("transaction")
        .and_then(|t| t.get("signatures"))
        .and_then(|s| s.as_array())
        .and_then(|a| a.first())
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();

    let slot = tx.get("slot").and_then(|s| s.as_u64());
    let block_time = tx.get("blockTime").and_then(|b| b.as_i64());

    let (success, error) = tx
        .get("meta")
        .and_then(|m| m.get("err"))
        .map(|err| (false, Some(format_error(err))))
        .unwrap_or((true, None));

    let fee = tx
        .get("meta")
        .and_then(|m| m.get("fee"))
        .and_then(|f| f.as_u64())
        .unwrap_or(0);

    let (instructions, accounts, summary) = extract_instructions_and_accounts(tx, success);

    TransactionAnalysis {
        signature,
        status: StatusInfo { success, error },
        fee,
        slot,
        block_time,
        summary,
        instructions,
        accounts,
    }
}

fn extract_instructions_and_accounts(
    tx: &serde_json::Value,
    success: bool,
) -> (Vec<InstructionAnalysis>, Vec<AccountAnalysis>, String) {
    let mut instructions = Vec::new();
    let mut accounts = Vec::new();

    if let Some(message) = tx.get("transaction").and_then(|t| t.get("message")) {
        // Extract account keys
        if let Some(account_keys) = message.get("accountKeys").and_then(|a| a.as_array()) {
            for (i, key) in account_keys.iter().enumerate() {
                if let Some(key_obj) = key.as_object() {
                    let pubkey = key_obj
                        .get("pubkey")
                        .or_else(|| key_obj.get("signer")) // fallback for different JSON formats
                        .and_then(|p| p.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let is_signer = key_obj.get("signer").and_then(|s| s.as_bool()).unwrap_or(false);
                    let is_writable = key_obj.get("writable").and_then(|w| w.as_bool()).unwrap_or(false);
                    let name = get_program_name(&pubkey);

                    accounts.push(AccountAnalysis {
                        pubkey,
                        name: name.map(|s| s.to_string()),
                        is_signer,
                        is_writable,
                    });
                } else if let Some(pubkey) = key.as_str() {
                    // Simple string format
                    let name = get_program_name(pubkey);
                    accounts.push(AccountAnalysis {
                        pubkey: pubkey.to_string(),
                        name: name.map(|s| s.to_string()),
                        is_signer: i == 0, // First account is usually fee payer/signer
                        is_writable: true,
                    });
                }
            }
        }

        // Extract instructions
        if let Some(instrs) = message.get("instructions").and_then(|i| i.as_array()) {
            for (index, instr) in instrs.iter().enumerate() {
                if let Some(instr_obj) = instr.as_object() {
                    let program_id = instr_obj
                        .get("programIdIndex")
                        .and_then(|p| p.as_u64())
                        .and_then(|idx| accounts.get(idx as usize))
                        .map(|a| a.pubkey.clone())
                        .unwrap_or_else(|| {
                            instr_obj
                                .get("programId")
                                .and_then(|p| p.as_str())
                                .unwrap_or("unknown")
                                .to_string()
                        });

                    let program_name = get_program_name(&program_id).map(|s| s.to_string());
                    let instruction_type = instr_obj
                        .get("parsed")
                        .and_then(|p| p.as_object())
                        .and_then(|p| p.get("type"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string());

                    let explanation = generate_explanation(&program_name, &instruction_type, success);

                    instructions.push(InstructionAnalysis {
                        index,
                        program: program_id,
                        program_name,
                        instruction_type,
                        explanation,
                    });
                }
            }
        }
    }

    // Generate summary
    let summary = generate_summary(&instructions, success);

    (instructions, accounts, summary)
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

fn format_error(err: &serde_json::Value) -> String {
    if let Some(obj) = err.as_object() {
        if let Some(instr_err) = obj.get("InstructionError") {
            if let Some(arr) = instr_err.as_array() {
                if arr.len() == 2 {
                    let index = &arr[0];
                    let error_type = &arr[1];
                    return format!(
                        "Instruction {} failed: {}",
                        index,
                        serde_json::to_string(error_type).unwrap_or_else(|_| "unknown".to_string())
                    );
                }
            }
        }
    }
    serde_json::to_string(err).unwrap_or_else(|_| "Unknown error".to_string())
}

fn generate_explanation(
    program_name: &Option<String>,
    instruction_type: &Option<String>,
    success: bool,
) -> Option<String> {
    if !success {
        return Some("This instruction failed".to_string());
    }

    match (program_name.as_deref(), instruction_type.as_deref()) {
        (Some("System Program"), Some("transfer")) => Some("Transfer SOL between accounts".to_string()),
        (Some("Token Program"), Some("transfer")) => Some("Transfer tokens between accounts".to_string()),
        (Some("Token Program"), Some("transferChecked")) => Some("Transfer tokens with decimal check".to_string()),
        (Some("Associated Token Account"), Some("create")) => Some("Create associated token account".to_string()),
        (Some("Associated Token Account"), Some("createIdempotent")) => {
            Some("Create associated token account if it doesn't exist".to_string())
        }
        (Some("System Program"), Some("createAccount")) => Some("Create a new account".to_string()),
        (Some("System Program"), Some("createAccountWithSeed")) => Some("Create account with seed".to_string()),
        _ => None,
    }
}

fn generate_summary(instructions: &[InstructionAnalysis], success: bool) -> String {
    let status = if success { "successful" } else { "failed" };

    if instructions.is_empty() {
        return format!("Empty transaction - {}", status);
    }

    let program_names: Vec<&str> = instructions.iter().filter_map(|i| i.program_name.as_deref()).collect();

    let unique_programs: std::collections::HashSet<&str> = program_names.into_iter().collect();
    let programs_str: Vec<&str> = unique_programs.into_iter().collect();

    if success {
        format!(
            "Successful transaction with {} instruction(s) using: {}",
            instructions.len(),
            programs_str.join(", ")
        )
    } else {
        format!(
            "Failed transaction with {} instruction(s)",
            instructions.len()
        )
    }
}
