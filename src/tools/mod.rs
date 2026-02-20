mod get_balance;
mod get_slot;
mod get_transaction;

pub use get_balance::*;
pub use get_slot::*;
pub use get_transaction::*;
use rust_mcp_sdk::tool_box;

tool_box!(
    SolanaTools,
    [GetBalanceTool, GetSlotTool, GetTransactionTool]
);
