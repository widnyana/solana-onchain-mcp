use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use solana_sdk::{message::Message, signature::Signer, transaction::Transaction};
use spl_token::instruction::revoke;

use crate::{error::Result, keypair::LoadedKeypair, rpc::SolanaRpcClient, utils::ParsePubkeyExt};

#[mcp_tool(
    name = "revoke_token",
    description = "Revoke a delegate's authority to spend tokens from your token account. \
Use this to remove approval previously granted via approve_token. \
This is recommended after completing DEX or lending protocol interactions to improve security."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct RevokeTokenTool {
    /// Token account to revoke delegate authority from
    pub token_account: String,
}

impl RevokeTokenTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient, keypair: &LoadedKeypair) -> Result<serde_json::Value> {
        let token_pubkey = self.token_account.parse_pubkey()?;

        let ix = revoke(
            &spl_token::id(),
            &token_pubkey,
            &keypair.keypair.pubkey(),
            &[],
        )
        .map_err(|e| {
            crate::error::SolanaMcpError::TransactionFailed(format!("Failed to create revoke instruction: {}", e))
        })?;

        let blockhash = client.get_latest_blockhash().await?;
        let message = Message::new(&[ix], Some(&keypair.keypair.pubkey()));
        let tx = Transaction::new(&[&keypair.keypair], message, blockhash);
        let sig = client.send_transaction(&tx).await?;

        Ok(serde_json::json!({
            "signature": sig.to_string(),
            "token_account": self.token_account,
        }))
    }
}
