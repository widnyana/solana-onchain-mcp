use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signer, transaction::Transaction};
use spl_token::instruction::close_account;

use crate::{error::Result, keypair::LoadedKeypair, rpc::SolanaRpcClient, utils::ParsePubkeyExt};

#[mcp_tool(
    name = "close_token_account",
    description = "Close an empty SPL token account and recover rent (~0.002 SOL). \
    The account must have zero balance. \
    Rent is returned to the destination account (usually your wallet)."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct CloseTokenAccountTool {
    /// Token account to close (must have 0 balance)
    pub token_account: String,
    /// Destination account to receive rent (usually your wallet address)
    pub destination: String,
}

impl CloseTokenAccountTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient, keypair: &LoadedKeypair) -> Result<serde_json::Value> {
        let token_pubkey = self.token_account.parse_pubkey()?;
        let dest_pubkey = self
            .destination
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(e.to_string()))?;

        let ix = close_account(
            &spl_token::id(),
            &token_pubkey,
            &dest_pubkey,
            &keypair.keypair.pubkey(),
            &[],
        )
        .map_err(|e| {
            crate::error::SolanaMcpError::TransactionFailed(format!("Failed to create close instruction: {}", e))
        })?;

        let blockhash = client.get_latest_blockhash().await?;
        let message = Message::new(&[ix], Some(&keypair.keypair.pubkey()));
        let tx = Transaction::new(&[&keypair.keypair], message, blockhash);
        let sig = client.send_transaction(&tx).await?;

        Ok(serde_json::json!({
            "signature": sig.to_string(),
            "token_account": self.token_account,
            "destination": self.destination,
        }))
    }
}
