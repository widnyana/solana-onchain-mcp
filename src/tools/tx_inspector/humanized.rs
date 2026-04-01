//! Humanized Transaction Inspector Tool
//!
//! Provides human-readable transaction analysis with explanations.

use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

use super::{decode_instruction, format_instruction_error, format_instruction_error_with_program, get_program_name};
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
            .await
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
    role: Option<String>,
    is_signer: bool,
    is_writable: bool,
}

/// Infer the role of an account based on its position and the instruction context
fn infer_account_role(
    index: usize,
    pubkey: &str,
    is_signer: bool,
    is_writable: bool,
    program_id: &str,
    instruction_type: Option<&str>,
) -> Option<String> {
    // First check if this is a known program
    if let Some(name) = get_program_name(pubkey) {
        return Some(name.to_string());
    }

    // Fee payer is always the first signer
    if is_signer && index == 0 {
        return Some("Fee Payer".to_string());
    }

    // Infer based on program and instruction type
    let system_program = "11111111111111111111111111111111";
    let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    let ata_program = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    match program_id {
        p if p == system_program => match instruction_type {
            Some("transfer") | Some("transferWithSeed") => {
                if index == 0 {
                    Some("Source".to_string())
                } else if index == 1 {
                    Some("Destination".to_string())
                } else {
                    None
                }
            }
            Some("createAccount") | Some("createAccountWithSeed") => {
                if index == 1 {
                    Some("New Account".to_string())
                } else if index == 2 {
                    Some("Owner".to_string())
                } else {
                    None
                }
            }
            Some("allocate") | Some("allocateWithSeed") => {
                if index == 0 {
                    Some("Account to Allocate".to_string())
                } else {
                    None
                }
            }
            Some("assign") | Some("assignWithSeed") => {
                if index == 0 {
                    Some("Account to Assign".to_string())
                } else {
                    None
                }
            }
            _ => None,
        },
        p if p == token_program => match instruction_type {
            Some("transfer") | Some("transferChecked") => {
                if is_writable && !is_signer {
                    Some("Token Account".to_string())
                } else {
                    None
                }
            }
            Some("mintTo") | Some("mintToChecked") => {
                if index == 1 {
                    Some("Destination Token Account".to_string())
                } else if index == 2 {
                    Some("Mint Authority".to_string())
                } else {
                    None
                }
            }
            Some("burn") | Some("burnChecked") => {
                if index == 0 {
                    Some("Token Account to Burn".to_string())
                } else {
                    None
                }
            }
            Some("initializeAccount") | Some("initializeAccount2") | Some("initializeAccount3") => {
                if index == 0 {
                    Some("New Token Account".to_string())
                } else if index == 1 {
                    Some("Mint".to_string())
                } else if index == 2 {
                    Some("Owner".to_string())
                } else {
                    None
                }
            }
            _ => None,
        },
        p if p == ata_program => match instruction_type {
            Some("create") | Some("createIdempotent") => {
                if index == 0 {
                    Some("Funding Account".to_string())
                } else if index == 1 {
                    Some("ATA Address".to_string())
                } else if index == 2 {
                    Some("Owner".to_string())
                } else if index == 3 {
                    Some("Mint".to_string())
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    }
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

    // Get raw error value first
    let raw_error = tx.get("meta").and_then(|m| m.get("err")).filter(|err| !err.is_null());

    let fee = tx
        .get("meta")
        .and_then(|m| m.get("fee"))
        .and_then(|f| f.as_u64())
        .unwrap_or(0);

    // Extract instructions and accounts
    let (instructions, accounts, summary) = extract_instructions_and_accounts(tx, raw_error);

    // Format error with program context if available
    let (success, error) = if let Some(err) = raw_error {
        // Try to get the failing instruction's program for better error interpretation
        let failing_program = get_failing_instruction_program(err, &instructions);
        let formatted = if let Some(program_id) = failing_program {
            format_instruction_error_with_program(err, &program_id)
        } else {
            format_instruction_error(err)
        };
        (false, Some(formatted))
    } else {
        (true, None)
    };

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

/// Get the program ID of the failing instruction from an InstructionError
fn get_failing_instruction_program(err: &serde_json::Value, instructions: &[InstructionAnalysis]) -> Option<String> {
    if let Some(obj) = err.as_object()
        && let Some(instr_err) = obj.get("InstructionError")
        && let Some(arr) = instr_err.as_array()
        && !arr.is_empty()
        && let Some(index) = arr[0].as_u64()
    {
        let idx = index as usize;
        return instructions.get(idx).map(|i| i.program.clone());
    }
    None
}

fn extract_instructions_and_accounts(
    tx: &serde_json::Value,
    _raw_error: Option<&serde_json::Value>,
) -> (Vec<InstructionAnalysis>, Vec<AccountAnalysis>, String) {
    let mut instructions = Vec::new();
    let mut accounts = Vec::new();

    if let Some(message) = tx.get("transaction").and_then(|t| t.get("message")) {
        // First pass: extract account keys (without roles yet)
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

                    accounts.push(AccountAnalysis {
                        pubkey: pubkey.clone(),
                        role: None, // Will be filled in second pass
                        is_signer,
                        is_writable,
                    });
                } else if let Some(pubkey) = key.as_str() {
                    // Simple string format
                    accounts.push(AccountAnalysis {
                        pubkey: pubkey.to_string(),
                        role: None,
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

                    // Get program name, or show "Unknown Program (truncated pubkey)" for unknown programs
                    let program_name = get_program_name(&program_id).or_else(|| {
                        // Truncate pubkey for display
                        let truncated = if program_id.len() > 12 {
                            format!("Unknown Program ({}...)", &program_id[..12])
                        } else {
                            format!("Unknown Program ({})", program_id)
                        };
                        Some(truncated)
                    });

                    // Try to get instruction type from parsed data first
                    let instruction_type = instr_obj
                        .get("parsed")
                        .and_then(|p| p.as_object())
                        .and_then(|p| p.get("type"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                        .or_else(|| {
                            // Fallback: try to decode instruction data
                            instr_obj
                                .get("data")
                                .and_then(|d| d.as_str())
                                .and_then(|data_b64| {
                                    // Decode base64 data
                                    use base64::{Engine, engine::general_purpose::STANDARD};
                                    STANDARD.decode(data_b64).ok()
                                })
                                .and_then(|data| {
                                    // Get account pubkeys for decode_instruction
                                    let account_pubkeys: Vec<String> =
                                        accounts.iter().map(|a| a.pubkey.clone()).collect();
                                    decode_instruction(&program_id, &data, &account_pubkeys)
                                        .ok()
                                        .map(|parsed| format!("{:?}", parsed))
                                })
                        });

                    let explanation = generate_explanation(&program_name, &instruction_type);

                    instructions.push(InstructionAnalysis {
                        index,
                        program: program_id.clone(),
                        program_name,
                        instruction_type,
                        explanation,
                    });

                    // Second pass: infer account roles for accounts involved in this instruction
                    if let Some(account_indices) = instr_obj.get("accounts").and_then(|a| a.as_array()) {
                        for acc_idx in account_indices {
                            if let Some(idx) = acc_idx.as_u64().and_then(|i| usize::try_from(i).ok())
                                && let Some(account) = accounts.get_mut(idx)
                                && account.role.is_none()
                            {
                                account.role = infer_account_role(
                                    idx,
                                    &account.pubkey,
                                    account.is_signer,
                                    account.is_writable,
                                    &program_id,
                                    instructions.last().and_then(|i| i.instruction_type.as_deref()),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Final pass: set role for any accounts that still don't have one
        for (i, account) in accounts.iter_mut().enumerate() {
            if account.role.is_none() {
                // Default role inference based on position and flags
                if account.is_signer && i == 0 {
                    account.role = Some("Fee Payer".to_string());
                } else if let Some(name) = get_program_name(&account.pubkey) {
                    account.role = Some(name.to_string());
                } else if account.is_writable {
                    account.role = Some("Account".to_string());
                } else {
                    account.role = Some("Read-only Account".to_string());
                }
            }
        }
    }

    // Generate summary (error details are in StatusInfo)
    let summary = generate_summary(&instructions);

    (instructions, accounts, summary)
}

fn generate_explanation(program_name: &Option<String>, instruction_type: &Option<String>) -> Option<String> {
    // Build the explanation based on program and instruction type
    let action = match (program_name.as_deref(), instruction_type.as_deref()) {
        (Some("System Program"), Some("transfer")) => "Transfer SOL between accounts",
        (Some("System Program"), Some("transferWithSeed")) => "Transfer SOL with seed",
        (Some("Token Program"), Some("transfer")) => "Transfer tokens between accounts",
        (Some("Token Program"), Some("transferChecked")) => "Transfer tokens with decimal check",
        (Some("Token Program"), Some("mintTo")) => "Mint new tokens",
        (Some("Token Program"), Some("mintToChecked")) => "Mint new tokens with check",
        (Some("Token Program"), Some("burn")) => "Burn tokens",
        (Some("Token Program"), Some("burnChecked")) => "Burn tokens with check",
        (Some("Token Program"), Some("initializeAccount")) => "Initialize token account",
        (Some("Token Program"), Some("initializeMint")) => "Initialize token mint",
        (Some("Token Program"), Some("closeAccount")) => "Close token account",
        (Some("Associated Token Account"), Some("create")) => "Create associated token account",
        (Some("Associated Token Account"), Some("createIdempotent")) => {
            "Create associated token account if it doesn't exist"
        }
        (Some("System Program"), Some("createAccount")) => "Create a new account",
        (Some("System Program"), Some("createAccountWithSeed")) => "Create account with seed",
        (Some("System Program"), Some("allocate")) => "Allocate space for account",
        (Some("System Program"), Some("assign")) => "Assign account to program",
        // Handle unknown programs with instruction type
        (Some(p), Some(t)) if p.starts_with("Unknown Program") => {
            return Some(format!("Execute {} instruction", t));
        }
        // Generic fallback
        (Some(p), Some(t)) => {
            return Some(format!("Execute {} on {}", t, p));
        }
        (Some(p), None) if p.starts_with("Unknown Program") => {
            return Some("Execute instruction on unknown program".to_string());
        }
        (Some(p), None) => {
            return Some(format!("Execute instruction on {}", p));
        }
        (None, Some(t)) => {
            return Some(format!("Execute {} instruction", t));
        }
        (None, None) => {
            return Some("Execute instruction".to_string());
        }
    };

    Some(action.to_string())
}

pub fn humanize_transaction_to_json(tx: &serde_json::Value) -> serde_json::Value {
    let analysis = create_humanized_analysis(tx);
    serde_json::to_value(analysis).unwrap_or(serde_json::Value::Null)
}

fn generate_summary(instructions: &[InstructionAnalysis]) -> String {
    if instructions.is_empty() {
        return "Empty transaction".to_string();
    }

    // Get unique programs
    let program_names: Vec<&str> = instructions.iter().filter_map(|i| i.program_name.as_deref()).collect();
    let unique_programs: std::collections::HashSet<&str> = program_names.into_iter().collect();
    let programs_str: Vec<&str> = unique_programs.into_iter().collect();

    // Get the main action from the first instruction
    let action = instructions
        .first()
        .and_then(|i| i.explanation.as_deref())
        .unwrap_or("execute instructions");

    // Build the summary
    let mut summary = format!("{} instruction(s) to {}", instructions.len(), action);

    if !programs_str.is_empty() {
        summary.push_str(&format!(" via {}", programs_str.join(", ")));
    }

    summary
}
