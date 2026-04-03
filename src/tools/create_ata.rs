use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use solana_sdk::{pubkey::Pubkey, signature::Signer, transaction::Transaction};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use crate::{error::Result, keypair::LoadedKeypair, rpc::SolanaRpcClient};

#[mcp_tool(
    name = "create_associated_token_account",
    description = "Create an Associated Token Account (ATA) for a token mint if it doesn't exist.

Use this tool before transfer_token when the recipient doesn't have a token account for that mint.
MUST Check first with `get_token_accounts_by_owner` to see if an ATA already exists.

Notice: This costs a small amount of SOL (rent-exempt balance) from your wallet."
)]
#[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct CreateAtaTool {
    /// The token mint address (e.g., USDC mint)
    pub token_mint: String,
    /// The owner address (defaults to your wallet if not specified)
    pub owner: Option<String>,
}

impl CreateAtaTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient, keypair: &LoadedKeypair) -> Result<serde_json::Value> {
        // Parse token mint
        let token_mint = self
            .token_mint
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(format!("Invalid token mint: {}", e)))?;

        // Get owner (default to keypair's pubkey)
        let owner = match &self.owner {
            Some(addr) => addr
                .parse::<Pubkey>()
                .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(format!("Invalid owner: {}", e)))?,
            None => keypair.keypair.pubkey(),
        };

        // Get ATA address
        let ata = get_associated_token_address(&owner, &token_mint);

        // Create ATA instruction
        let create_ata_ix = create_associated_token_account(
            &keypair.keypair.pubkey(),
            &owner,
            &token_mint,
            &spl_token::ID,
        );

        // Get latest blockhash and build transaction
        let recent_hash = client.get_latest_blockhash().await?;
        let transaction = Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&keypair.keypair.pubkey()),
            &[&keypair.keypair],
            recent_hash,
        );

        // Send transaction
        let signature = client.send_transaction(&transaction).await?;

        Ok(serde_json::json!({
            "signature": signature.to_string(),
            "ata": ata.to_string(),
            "token_mint": self.token_mint,
            "owner": owner.to_string(),
        }))
    }
}
