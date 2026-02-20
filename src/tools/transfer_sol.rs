use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
#[allow(deprecated)]
use solana_sdk::system_instruction;
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signer, transaction::Transaction};

use crate::{keypair::LoadedKeypair, rpc::SolanaRpcClient};

#[mcp_tool(
    name = "transfer_sol",
    description = "Transfer SOL from the configured wallet to a recipient address. \
    The amount is specified in lamports (1 SOL = 1,000,000,000 lamports). \
    Use this tool when the user wants to send SOL to another address. \
    Requires a configured keypair (SOLANA_KEYPAIR_PATH env variable)."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct TransferSolTool {
    /// The recipient Solana address (base58 encoded, 32-44 characters)
    pub to_address: String,
    /// Amount to transfer in lamports (1 SOL = 1,000,000,000 lamports)
    pub amount_lamports: u64,
}

impl TransferSolTool {
    pub async fn call_tool(
        &self,
        client: &SolanaRpcClient,
        keypair: &LoadedKeypair,
    ) -> Result<serde_json::Value, crate::error::SolanaMcpError> {
        // Parse recipient address
        let to_pubkey = self
            .to_address
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(e.to_string()))?;

        // Create transfer instruction
        #[allow(deprecated)]
        let instruction = system_instruction::transfer(&keypair.keypair.pubkey(), &to_pubkey, self.amount_lamports);

        // Get latest blockhash
        let blockhash = client.get_latest_blockhash().await?;

        // Build message and transaction
        let message = Message::new(&[instruction], Some(&keypair.keypair.pubkey()));
        let transaction = Transaction::new(&[&keypair.keypair], message, blockhash);

        // Send transaction
        let signature = client.send_transaction(&transaction).await?;

        Ok(serde_json::json!({
            "signature": signature.to_string(),
            "amount_lamports": self.amount_lamports,
            "from": keypair.pubkey,
            "to": self.to_address,
            "amount_sol": self.amount_lamports as f64 / 1_000_000_000.0,
        }))
    }
}
