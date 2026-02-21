//! Transaction Inspector Tools
//!
//! This module provides tools for inspecting and analyzing Solana transactions.

mod core;
mod humanized;
mod programs;
mod raw;

pub use humanized::InspectTransactionHumanizedTool;
pub use raw::InspectTransactionRawTool;

// Public exports for enhanced transaction parsing (future use)
#[allow(unused_imports, dead_code)]
pub use core::{
    AccountRef, AccountRole, BalanceChange, ParseConfig, ParsedInstruction,
    ParsedInstructionData, ParseError, ParsedTransaction, TokenBalanceChange, TransactionSummary,
};
#[allow(unused_imports, dead_code)]
pub use programs::{
    decode_instruction, identify_program, interpret_error, ProgramCategory, ProgramInfo,
    ProgramRegistry,
};

/// Lookup table for well-known Solana program names
pub(crate) fn get_program_name(pubkey: &str) -> Option<&'static str> {
    identify_program(pubkey).map(|info| {
        // The ProgramInfo is static, so we can return a static str
        let name: &str = &info.name;
        // SAFETY: ProgramInfo instances are lazily static, names are &'static str
        unsafe { std::mem::transmute::<&str, &'static str>(name) }
    })
}

/// Format an InstructionError into a human-readable string
pub(crate) fn format_instruction_error(err: &serde_json::Value) -> String {
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
