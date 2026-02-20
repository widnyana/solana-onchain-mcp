use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use solana_sdk::{message::Message, pubkey::Pubkey, signature::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer_checked;

use crate::{error::Result, keypair::LoadedKeypair, rpc::SolanaRpcClient};

#[mcp_tool(
    name = "transfer_token",
    description = "Transfer SPL tokens from the configured wallet to a recipient address. \
    Use this when the user wants to send SPL tokens (like USDC, USDT, or other fungible tokens) \
    to another Solana address. The keypair must be configured via SOLANA_KEYPAIR_PATH environment variable."
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct TransferTokenTool {
    /// The recipient's Solana address (base58 encoded)
    pub to_address: String,
    /// The token mint address (e.g., USDC mint on devnet/mainnet)
    pub token_mint: String,
    /// The amount of tokens to transfer in UI units (e.g., 1.5 for 1.5 tokens)
    pub amount: f64,
    /// The number of decimals for the token (e.g., 6 for USDC, 9 for native-wrapped SOL)
    pub decimals: u8,
}

impl TransferTokenTool {
    pub async fn call_tool(&self, client: &SolanaRpcClient, keypair: &LoadedKeypair) -> Result<serde_json::Value> {
        // Validate decimals for non-integer amounts
        if self.amount.fract() != 0.0 && self.decimals == 0 {
            return Err(crate::error::SolanaMcpError::InvalidTokenAccount(
                "Token has 0 decimals but non-integer amount provided".to_string(),
            ));
        }

        // Parse addresses
        let to_pubkey = self
            .to_address
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(e.to_string()))?;

        let token_mint = self
            .token_mint
            .parse::<Pubkey>()
            .map_err(|e| crate::error::SolanaMcpError::InvalidAddress(format!("Invalid token mint: {}", e)))?;

        let from_pubkey = keypair.keypair.pubkey();

        // Calculate raw amount: (amount * 10^decimals) as u64
        let multiplier = 10_f64.powi(self.decimals as i32);
        let raw_amount = (self.amount * multiplier) as u64;

        // Get ATAs for both sender and recipient
        let from_ata = get_associated_token_address(&from_pubkey, &token_mint);
        let to_ata = get_associated_token_address(&to_pubkey, &token_mint);

        // Get latest blockhash
        let recent_hash = client.get_latest_blockhash().await?;

        // Create transfer_checked instruction
        let transfer_ix = transfer_checked(
            &spl_token::ID,
            &from_ata,
            &token_mint,
            &to_ata,
            &from_pubkey,
            &[],
            raw_amount,
            self.decimals,
        )
        .map_err(|e| {
            crate::error::SolanaMcpError::TransactionFailed(format!("Failed to create transfer instruction: {}", e))
        })?;

        // Build transaction
        let message = Message::new(&[transfer_ix], Some(&from_pubkey));
        let transaction = Transaction::new(&[&keypair.keypair], message, recent_hash);

        // Send transaction
        let signature = client.send_transaction(&transaction).await?;

        Ok(serde_json::json!({
            "signature": signature.to_string(),
            "token_mint": self.token_mint,
            "amount": self.amount,
            "decimals": self.decimals,
            "raw_amount": raw_amount,
            "from": from_pubkey.to_string(),
            "from_ata": from_ata.to_string(),
            "to": self.to_address,
            "to_ata": to_ata.to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    /// Helper function to check if decimal validation would fail
    fn validate_decimals(amount: f64, decimals: u8) -> bool {
        amount.fract() != 0.0 && decimals == 0
    }

    /// Helper function to calculate raw amount (mirrors the logic in call_tool)
    fn calculate_raw_amount(amount: f64, decimals: u8) -> u64 {
        let multiplier = 10_f64.powi(decimals as i32);
        (amount * multiplier) as u64
    }

    #[test]
    fn test_decimal_validation_zero_with_fraction_fails() {
        // amount=1.5 with decimals=0 should trigger the validation error
        let amount = 1.5_f64;
        let decimals = 0_u8;

        assert!(
            validate_decimals(amount, decimals),
            "Expected validation to fail for amount=1.5 with decimals=0"
        );
    }

    #[test]
    fn test_decimal_validation_zero_with_integer_succeeds() {
        // amount=2.0 with decimals=0 should NOT trigger the error (fract() == 0.0)
        let amount = 2.0_f64;
        let decimals = 0_u8;

        assert!(
            !validate_decimals(amount, decimals),
            "Expected validation to pass for amount=2.0 with decimals=0"
        );
    }

    #[test]
    fn test_raw_amount_calculation() {
        // 1.5 USDC (decimals=6) should calculate raw_amount = 1_500_000
        let amount = 1.5_f64;
        let decimals = 6_u8;

        let raw_amount = calculate_raw_amount(amount, decimals);

        assert_eq!(
            raw_amount, 1_500_000,
            "Expected 1.5 USDC with 6 decimals to equal 1,500,000 raw units"
        );
    }

    #[test]
    fn test_raw_amount_zero() {
        // amount=0.0 should produce raw_amount=0
        let amount = 0.0_f64;
        let decimals = 6_u8;

        let raw_amount = calculate_raw_amount(amount, decimals);

        assert_eq!(raw_amount, 0, "Expected amount=0.0 to produce raw_amount=0");
    }
}
