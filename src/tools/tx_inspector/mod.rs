//! Transaction Inspector Tools
//!
//! This module provides tools for inspecting and analyzing Solana transactions.

mod humanized;
mod raw;

pub use humanized::InspectTransactionHumanizedTool;
pub use raw::InspectTransactionRawTool;

/// Lookup table for well-known Solana program names
pub(crate) fn get_program_name(pubkey: &str) -> Option<&'static str> {
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
