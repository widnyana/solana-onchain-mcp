use rust_mcp_sdk::{
    schema::{CallToolResult, TextContent},
    tool_box,
};

mod approve_token;
mod create_ata;
mod get_account_info;
mod get_balance;
mod get_multiple_accounts;
mod get_program_accounts;
mod get_server_info;
mod get_signature_status;
mod get_signatures_for_address;
mod get_slot;
mod get_token_accounts_by_owner;
mod get_transaction;
mod revoke_token;
mod simulate_transaction;
mod transfer_sol;
mod transfer_token;
pub mod tx_inspector;

pub use approve_token::*;
pub use create_ata::*;
pub use get_account_info::*;
pub use get_balance::*;
pub use get_multiple_accounts::*;
pub use get_program_accounts::*;
pub use get_server_info::*;
pub use get_signature_status::*;
pub use get_signatures_for_address::*;
pub use get_slot::*;
pub use get_token_accounts_by_owner::*;
pub use get_transaction::*;
pub use revoke_token::*;
pub use simulate_transaction::*;
pub use transfer_sol::*;
pub use transfer_token::*;
pub use tx_inspector::{
    InspectTransactionHumanizedTool, InspectTransactionRawTool, ParsedInstructionData, ProgramCategory,
    decode_instruction, format_instruction_error, get_program_name, identify_program, interpret_error,
};

tool_box!(
    SolanaTools,
    [
        ApproveTokenTool,
        CreateAtaTool,
        GetAccountInfoTool,
        GetBalanceTool,
        GetMultipleAccountsTool,
        GetProgramAccountsTool,
        GetServerInfoTool,
        GetSignaturesForAddressTool,
        GetSignatureStatusTool,
        GetSlotTool,
        GetTokenAccountsByOwnerTool,
        GetTransactionTool,
        RevokeTokenTool,
        SimulateTransactionTool,
        TransferSolTool,
        TransferTokenTool,
        InspectTransactionRawTool,
        InspectTransactionHumanizedTool
    ]
);

/// Helper to create a JSON result from a serializable value.
/// Falls back to the provided string if serialization fails.
pub fn json_result<T: serde::Serialize>(value: T, fallback: &str) -> CallToolResult {
    CallToolResult::text_content(vec![TextContent::from(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| fallback.to_string()),
    )])
}

/// Helper to create a text result from a message.
pub fn text_result(msg: impl Into<String>) -> CallToolResult {
    CallToolResult::text_content(vec![TextContent::from(msg.into())])
}

/// Serialize a value to JSON text for MCP responses.
pub fn json_to_text<T: serde::Serialize>(value: &T) -> Result<TextContent, serde_json::Error> {
    serde_json::to_string(value).map(TextContent::from)
}
