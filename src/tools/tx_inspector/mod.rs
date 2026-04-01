//! Transaction Inspector Tools
//!
//! This module provides tools for inspecting and analyzing Solana transactions.

mod core;
mod humanized;
mod programs;
mod raw;

// Public exports for enhanced transaction parsing (future use)
#[allow(unused_imports, dead_code)]
pub use core::{
    AccountRef, AccountRole, BalanceChange, ParseConfig, ParseError, ParsedInstruction, ParsedInstructionData,
    ParsedTransaction, TokenBalanceChange, TransactionSummary,
};

pub use humanized::{InspectTransactionHumanizedTool, humanize_transaction_to_json};
#[allow(unused_imports, dead_code)]
pub use programs::{
    ProgramCategory, ProgramInfo, ProgramRegistry, decode_instruction, identify_program, interpret_error,
};
pub use raw::InspectTransactionRawTool;

/// Lookup table for well-known Solana program names
pub fn get_program_name(pubkey: &str) -> Option<String> {
    identify_program(pubkey).map(|info| info.name.clone())
}

/// Format an InstructionError into a human-readable string
pub fn format_instruction_error(err: &serde_json::Value) -> String {
    if let Some(obj) = err.as_object()
        && let Some(instr_err) = obj.get("InstructionError")
        && let Some(arr) = instr_err.as_array()
        && arr.len() == 2
    {
        let index = &arr[0];
        let error_type = &arr[1];
        return format!(
            "Instruction {} failed: {}",
            index,
            serde_json::to_string(error_type).unwrap_or_else(|_| "unknown error".to_string())
        );
    }
    serde_json::to_string(err).unwrap_or_else(|_| "unknown error".to_string())
}

/// Format an InstructionError with program-specific error interpretation
pub fn format_instruction_error_with_program(err: &serde_json::Value, program_id: &str) -> String {
    if let Some(obj) = err.as_object()
        && let Some(instr_err) = obj.get("InstructionError")
        && let Some(arr) = instr_err.as_array()
        && arr.len() == 2
    {
        let index = &arr[0];
        let error_type = &arr[1];

        // Try to interpret Custom error codes
        if let Some(error_obj) = error_type.as_object()
            && let Some(custom_code) = error_obj.get("Custom").and_then(|c| c.as_u64())
        {
            let interpreted = interpret_error(program_id, custom_code as u32);
            return format!("Instruction {} failed: {}", index, interpreted);
        }

        return format!(
            "Instruction {} failed: {}",
            index,
            serde_json::to_string(error_type).unwrap_or_else(|_| "unknown error".to_string())
        );
    }
    serde_json::to_string(err).unwrap_or_else(|_| "unknown error".to_string())
}

/// Apply mint/burn/transfer flags from an instruction_type string.
fn apply_instruction_type_flags(instruction_type: &str, has_mint: &mut bool, has_burn: &mut bool, has_transfer: &mut bool) {
    let it_lower = instruction_type.to_lowercase();
    if it_lower.contains("mintto") {
        *has_mint = true;
    } else if it_lower.contains("burn") {
        *has_burn = true;
    } else if it_lower.contains("transfer") {
        *has_transfer = true;
    }
}

/// Classify a humanized transaction (from `humanize_transaction_to_json`) into a transaction type.
///
/// Taxonomy (first match wins, DeFi checked before NFT):
/// - Any instruction with a DeFi program → "swap"
/// - Any instruction with an NFT program → "nft"
/// - All Core programs; any instruction_type contains "mintTo" or "MintTo" → "mint"
/// - All Core programs; any instruction_type contains "burn" or "Burn" → "burn"
/// - All Core programs; any instruction_type contains "transfer" or "Transfer" → "transfer"
/// - Otherwise → "unknown"
pub fn classify_tx_type(humanized: &serde_json::Value) -> &'static str {
    let instructions = match humanized.get("instructions").and_then(|i| i.as_array()) {
        Some(arr) => arr,
        None => return "unknown",
    };

    let mut has_defi = false;
    let mut has_nft = false;
    let mut has_mint = false;
    let mut has_burn = false;
    let mut has_transfer = false;

    for instr in instructions {
        let program = instr.get("program").and_then(|p| p.as_str()).unwrap_or("");
        let instruction_type = instr
            .get("instruction_type")
            .and_then(|t| t.as_str())
            .unwrap_or("");

        if let Some(info) = identify_program(program) {
            match info.category {
                ProgramCategory::DeFi => has_defi = true,
                ProgramCategory::NFT => has_nft = true,
                ProgramCategory::Core => {
                    apply_instruction_type_flags(instruction_type, &mut has_mint, &mut has_burn, &mut has_transfer);
                }
                _ => {}
            }
        } else {
            // Unknown program — check instruction_type strings anyway
            apply_instruction_type_flags(instruction_type, &mut has_mint, &mut has_burn, &mut has_transfer);
        }
    }

    if has_defi {
        "swap"
    } else if has_nft {
        "nft"
    } else if has_mint {
        "mint"
    } else if has_burn {
        "burn"
    } else if has_transfer {
        "transfer"
    } else {
        "unknown"
    }
}
