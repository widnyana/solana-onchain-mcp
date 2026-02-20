use std::{sync::Arc, time::Duration};

use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use tokio::task::spawn_blocking;

use crate::{
    config::Config,
    error::{Result, SolanaMcpError},
};

#[derive(Clone)]
pub struct SolanaRpcClient {
    client: Arc<RpcClient>,
}

impl SolanaRpcClient {
    pub fn new(config: &Config) -> Self {
        let client = RpcClient::new_with_timeout_and_commitment(
            config.rpc_url.clone(),
            Duration::from_secs(30),
            CommitmentConfig::confirmed(),
        );

        Self { client: Arc::new(client) }
    }

    pub fn parse_commitment(commitment: Option<&str>) -> CommitmentConfig {
        match commitment {
            Some("processed") => CommitmentConfig::processed(),
            Some("finalized") => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        }
    }

    pub fn get_balance(&self, address: &str, commitment: Option<&str>) -> Result<u64> {
        let pubkey = address
            .parse::<Pubkey>()
            .map_err(|e| SolanaMcpError::InvalidAddress(e.to_string()))?;

        let commitment_config = Self::parse_commitment(commitment);
        let response = self
            .client
            .get_balance_with_commitment(&pubkey, commitment_config)
            .map_err(SolanaMcpError::from)?;

        Ok(response.value)
    }

    pub fn get_slot(&self, commitment: Option<&str>) -> Result<u64> {
        let commitment_config = Self::parse_commitment(commitment);
        let slot = self
            .client
            .get_slot_with_commitment(commitment_config)
            .map_err(SolanaMcpError::from)?;

        Ok(slot)
    }

    pub fn get_transaction(&self, signature: &str, commitment: Option<&str>) -> Result<serde_json::Value> {
        let sig = signature
            .parse::<Signature>()
            .map_err(|e| SolanaMcpError::InvalidSignature(e.to_string()))?;

        let commitment_config = Self::parse_commitment(commitment);
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: Some(commitment_config),
            max_supported_transaction_version: Some(0),
        };

        let tx = self
            .client
            .get_transaction_with_config(&sig, config)
            .map_err(SolanaMcpError::from)?;

        Ok(serde_json::to_value(tx).unwrap_or(serde_json::json!({
            "error": "Failed to serialize transaction"
        })))
    }

    pub async fn get_latest_blockhash(&self) -> Result<Hash> {
        let client = self.clone();
        spawn_blocking(move || client.client.get_latest_blockhash())
            .await
            .map_err(|_| SolanaMcpError::TaskJoinError)?
            .map_err(SolanaMcpError::from)
    }

    pub async fn send_transaction(&self, tx: &Transaction) -> Result<Signature> {
        let client = self.clone();
        let tx = tx.clone();
        spawn_blocking(move || client.client.send_transaction(&tx))
            .await
            .map_err(|_| SolanaMcpError::TaskJoinError)?
            .map_err(SolanaMcpError::from)
    }

    pub async fn confirm_transaction(&self, sig: &Signature) -> Result<bool> {
        let client = self.clone();
        let sig = *sig;
        spawn_blocking(move || client.client.confirm_transaction(&sig).map_err(SolanaMcpError::from))
            .await
            .map_err(|_| SolanaMcpError::TaskJoinError)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commitment_none_defaults_to_confirmed() {
        let result = SolanaRpcClient::parse_commitment(None);
        assert_eq!(result, CommitmentConfig::confirmed());
    }

    #[test]
    fn test_parse_commitment_processed() {
        let result = SolanaRpcClient::parse_commitment(Some("processed"));
        assert_eq!(result, CommitmentConfig::processed());
    }

    #[test]
    fn test_parse_commitment_finalized() {
        let result = SolanaRpcClient::parse_commitment(Some("finalized"));
        assert_eq!(result, CommitmentConfig::finalized());
    }

    #[test]
    fn test_parse_commitment_invalid_defaults() {
        let result = SolanaRpcClient::parse_commitment(Some("invalid"));
        assert_eq!(result, CommitmentConfig::confirmed());
    }

    #[test]
    fn test_parse_commitment_case_sensitive() {
        // "Processed" with capital P should default to confirmed (case sensitive)
        let result = SolanaRpcClient::parse_commitment(Some("Processed"));
        assert_eq!(result, CommitmentConfig::confirmed());
    }
}
