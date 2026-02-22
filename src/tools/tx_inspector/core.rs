//! Transaction Inspector Core Module
//!
//! This module defines the core data structures for parsing and representing
//! Solana transactions.

use serde::{Deserialize, Serialize};

/// Configuration for parsing transactions
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParseConfig {
    /// Whether to human-readable values (e.g., SOL instead of lamports)
    pub humanize: bool,
    /// Whether to include transaction summary
    pub include_summary: bool,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self { humanize: true, include_summary: true }
    }
}

/// Parsed instruction data - either raw (hex) or decoded (JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParsedInstructionData {
    Raw(String),
    Decoded(serde_json::Value),
}

/// Errors that can occur during transaction parsing
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ParseError {
    #[error("Failed to parse transaction: {0}")]
    ParseFailed(String),

    #[error("Unknown program: {0}")]
    UnknownProgram(String),

    #[error("Invalid instruction data: {0}")]
    InvalidInstructionData(String),

    #[error("Program error: {code}: {message}")]
    ProgramError { code: u32, message: String },
}

/// A parsed transaction with all instructions decoded
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ParsedTransaction {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub fee: u64,
    pub success: bool,
    pub error: Option<String>,
    pub instructions: Vec<ParsedInstruction>,
    pub summary: Option<TransactionSummary>,
}

/// A single parsed instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ParsedInstruction {
    pub program: String,
    pub program_name: String,
    pub instruction: String,
    pub description: String,
    pub accounts: Vec<AccountRef>,
    pub data: ParsedInstructionData,
}

/// Reference to an account in an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AccountRef {
    pub pubkey: String,
    pub role: AccountRole,
    pub writable: bool,
    pub signer: bool,
}

/// The role an account plays in an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    /// Account pays for the transaction
    Payer,
    /// Account that will own the new account
    Owner,
    /// Account being created
    New,
    /// System program
    System,
    /// Token mint
    Mint,
    /// Token account
    Token,
    /// Associated token account
    ATA,
    /// Authority/account owner
    Authority,
    /// Destination account
    Destination,
    /// Source account
    Source,
    /// Another program account
    Program,
    /// Unknown role
    Unknown,
}

/// Summary of transaction effects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TransactionSummary {
    pub native_balance_changes: Vec<BalanceChange>,
    pub token_balance_changes: Vec<TokenBalanceChange>,
    pub accounts_created: Vec<String>,
    pub accounts_closed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BalanceChange {
    pub account: String,
    pub change: i64,
    pub post_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TokenBalanceChange {
    pub account: String,
    pub mint: String,
    pub change: i64,
    pub decimals: u8,
    pub post_balance: u64,
}
