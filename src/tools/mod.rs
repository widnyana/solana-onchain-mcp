mod get_account_info;
mod get_balance;
mod get_multiple_accounts;
mod get_program_accounts;
mod get_signatures_for_address;
mod get_slot;
mod get_token_accounts_by_owner;
mod get_transaction;
mod simulate_transaction;
mod transfer_sol;
mod transfer_token;

pub use get_account_info::*;
pub use get_balance::*;
pub use get_multiple_accounts::*;
pub use get_program_accounts::*;
pub use get_signatures_for_address::*;
pub use get_slot::*;
pub use get_token_accounts_by_owner::*;
pub use get_transaction::*;
use rust_mcp_sdk::tool_box;
pub use simulate_transaction::*;
pub use transfer_sol::*;
pub use transfer_token::*;

tool_box!(
    SolanaTools,
    [
        GetAccountInfoTool,
        GetBalanceTool,
        GetMultipleAccountsTool,
        GetProgramAccountsTool,
        GetSignaturesForAddressTool,
        GetSlotTool,
        GetTokenAccountsByOwnerTool,
        GetTransactionTool,
        SimulateTransactionTool,
        TransferSolTool,
        TransferTokenTool
    ]
);

/// Serialize a value to JSON text for MCP responses.
/// Uses compact JSON (not pretty) for better performance.
pub fn json_to_text<T: serde::Serialize>(value: &T) -> Result<TextContent, serde_json::Error> {
    serde_json::to_string(value).map(TextContent::from)
}
