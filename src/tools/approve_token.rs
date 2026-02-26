use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signer, transaction::Transaction};
use spl_token::instruction::approve;

use crate::{error::Result, keypair::LoadedKeypair, rpc::SolanaRpcClient, utils::ParsePubkeyExt};

#[mcp_tool(
    name = "approve_token",
    description = "[IRREVERSIBLE] Approve a delegate to spend tokens from your token account. \
    Required for DEX and lending protocol interactions. \
    The delegate can transfer up to the approved amount. \
    Revoke with revoke_token when no longer needed."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct ApproveTokenTool {
    /// Token account to approve spending from
    pub token_account: String,
    /// Delegate address (usually a program or protocol)
    pub delegate: String,
    /// Maximum amount delegate can spend (in token base units)
    pub amount: u64,
}

impl ApproveTokenTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient, keypair: &LoadedKeypair) -> Result<serde_json::Value> {
        let token_pubkey = self.token_account.parse_pubkey()?;
        let delegate_pubkey = self
            .delegate
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(e.to_string()))?;

        let ix = approve(
            &spl_token::id(),
            &token_pubkey,
            &delegate_pubkey,
            &keypair.keypair.pubkey(),
            &[],
            self.amount,
        )
        .map_err(|e| {
            crate::error::SolanaMcpError::TransactionFailed(format!("Failed to create approve instruction: {}", e))
        })?;

        let blockhash = client.get_latest_blockhash().await?;
        let message = Message::new(&[ix], Some(&keypair.keypair.pubkey()));
        let tx = Transaction::new(&[&keypair.keypair], message, blockhash);
        let sig = client.send_transaction(&tx).await?;

        Ok(serde_json::json!({
            "signature": sig.to_string(),
            "token_account": self.token_account,
            "delegate": self.delegate,
            "amount": self.amount,
        }))
    }
}
