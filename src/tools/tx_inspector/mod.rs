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

pub use humanized::InspectTransactionHumanizedTool;
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
