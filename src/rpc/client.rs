use std::{sync::Arc, time::Duration};

use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

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
}
