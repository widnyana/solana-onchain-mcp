mod get_balance;
mod get_slot;
mod get_transaction;
mod transfer_sol;
mod transfer_token;

pub use get_balance::*;
pub use get_slot::*;
pub use get_transaction::*;
use rust_mcp_sdk::tool_box;
pub use transfer_sol::*;
pub use transfer_token::*;

tool_box!(
    SolanaTools,
    [
        GetBalanceTool,
        GetSlotTool,
        GetTransactionTool,
        TransferSolTool,
        TransferTokenTool
    ]
);
