use std::{sync::Arc, time::Duration};

use base64::{Engine, prelude::BASE64_STANDARD};
use bincode::deserialize;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
    rpc_config::{
        RpcAccountInfoConfig, RpcProgramAccountsConfig, RpcSendTransactionConfig, RpcSimulateTransactionConfig,
        RpcTransactionConfig,
    },
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    rpc_request::TokenAccountsFilter,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::Hash, pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};
use tokio::task::spawn_blocking;

use crate::{
    config::Config,
    error::{Result, SolanaMcpError},
    utils::ParsePubkeyExt,
};

#[derive(Clone)]
pub struct SolanaRpcClient {
    client: Arc<RpcClient>,
}

impl SolanaRpcClient {
    const MAX_MULTIPLE_ACCOUNTS: usize = 100;
    const MAX_TRANSACTION_INPUT_SIZE: usize = 4096;

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

    fn parse_encoding(encoding: &str) -> Result<UiAccountEncoding> {
        match encoding {
            "base58" => Ok(UiAccountEncoding::Base58),
            "base64" => Ok(UiAccountEncoding::Base64),
            "jsonParsed" => Ok(UiAccountEncoding::JsonParsed),
            other => Err(SolanaMcpError::InvalidEncoding(other.to_string())),
        }
    }

    fn encode_account_data(data: &[u8], encoding: UiAccountEncoding) -> serde_json::Value {
        match encoding {
            UiAccountEncoding::Base58 => {
                serde_json::json!({
                    "data": bs58::encode(data).into_string(),
                    "encoding": "base58"
                })
            }
            UiAccountEncoding::JsonParsed => {
                // For jsonParsed, fall back to base64 as we can't parse arbitrary data
                serde_json::json!({
                    "data": BASE64_STANDARD.encode(data),
                    "encoding": "base64"
                })
            }
            _ => {
                serde_json::json!({
                    "data": BASE64_STANDARD.encode(data),
                    "encoding": "base64"
                })
            }
        }
    }

    fn serialize_account(account: solana_sdk::account::Account, encoding: UiAccountEncoding) -> serde_json::Value {
        let data_encoded = Self::encode_account_data(&account.data, encoding);
        serde_json::json!({
            "lamports": account.lamports,
            "owner": account.owner.to_string(),
            "executable": account.executable,
            "rent_epoch": account.rent_epoch,
            "data": data_encoded,
            "space": account.data.len(),
        })
    }

    pub fn get_balance(&self, address: &str, commitment: Option<&str>) -> Result<u64> {
        let pubkey = address.parse_pubkey()?;

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

    // ==================== NEW READ TOOLS ====================

    /// Get transaction with specific encoding, returning the full typed response
    pub fn get_transaction_with_config(
        &self,
        signature: &Signature,
        commitment: Option<&str>,
        encoding: UiTransactionEncoding,
    ) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let commitment_config = Self::parse_commitment(commitment);
        let config = RpcTransactionConfig {
            encoding: Some(encoding),
            commitment: Some(commitment_config),
            max_supported_transaction_version: Some(0),
        };

        self.client
            .get_transaction_with_config(signature, config)
            .map_err(SolanaMcpError::from)
    }

    pub fn get_account_info(
        &self,
        address: &str,
        encoding: Option<&str>,
        commitment: Option<&str>,
    ) -> Result<Option<serde_json::Value>> {
        let pubkey = address.parse_pubkey()?;

        let enc = Self::parse_encoding(encoding.unwrap_or("base64"))?;
        let config = RpcAccountInfoConfig {
            encoding: Some(enc),
            commitment: Some(Self::parse_commitment(commitment)),
            data_slice: None,
            min_context_slot: None,
        };

        let response = self
            .client
            .get_account_with_config(&pubkey, config)
            .map_err(SolanaMcpError::from)?;

        Ok(response.value.map(|acc| Self::serialize_account(acc, enc)))
    }

    pub fn get_multiple_accounts(
        &self,
        addresses: &[String],
        encoding: Option<&str>,
        commitment: Option<&str>,
    ) -> Result<Vec<Option<serde_json::Value>>> {
        if addresses.len() > Self::MAX_MULTIPLE_ACCOUNTS {
            return Err(SolanaMcpError::RpcError(format!(
                "Maximum {} addresses allowed per request",
                Self::MAX_MULTIPLE_ACCOUNTS
            )));
        }

        let pubkeys: Vec<Pubkey> = addresses
            .iter()
            .map(|addr| addr.parse_pubkey())
            .collect::<Result<Vec<_>>>()?;

        let enc = Self::parse_encoding(encoding.unwrap_or("base64"))?;
        let config = RpcAccountInfoConfig {
            encoding: Some(enc),
            commitment: Some(Self::parse_commitment(commitment)),
            data_slice: None,
            min_context_slot: None,
        };

        let response = self
            .client
            .get_multiple_accounts_with_config(&pubkeys, config)
            .map_err(SolanaMcpError::from)?;

        Ok(response
            .value
            .into_iter()
            .map(|opt_acc| opt_acc.map(|acc| Self::serialize_account(acc, enc)))
            .collect())
    }

    pub fn get_token_accounts_by_owner(
        &self,
        owner_address: &str,
        mint: Option<&str>,
        program_id: Option<&str>,
        commitment: Option<&str>,
    ) -> Result<serde_json::Value> {
        let _commitment_config = Self::parse_commitment(commitment); // Parsed but not yet used - reserved for future implementation
        let owner = owner_address.parse_pubkey()?;

        let token_account_filter = match (mint, program_id) {
            (Some(m), None) => TokenAccountsFilter::Mint(
                m.parse()
                    .map_err(|e| SolanaMcpError::InvalidAddress(format!("Invalid mint address: {}", e)))?,
            ),
            (None, Some(p)) => TokenAccountsFilter::ProgramId(
                p.parse()
                    .map_err(|e| SolanaMcpError::InvalidAddress(format!("Invalid program_id: {}", e)))?,
            ),
            _ => {
                return Err(SolanaMcpError::RpcError(
                    "Must specify mint or program_id".to_string(),
                ));
            }
        };

        let response = self
            .client
            .get_token_accounts_by_owner(&owner, token_account_filter)
            .map_err(SolanaMcpError::from)?;

        Ok(serde_json::to_value(response).unwrap_or(serde_json::json!([])))
    }

    pub fn get_signatures_for_address(
        &self,
        address: &str,
        limit: Option<usize>,
        before: Option<&str>,
        until: Option<&str>,
        commitment: Option<&str>,
    ) -> Result<serde_json::Value> {
        let pubkey = address.parse_pubkey()?;

        let before_sig = before
            .map(|s| s.parse::<Signature>())
            .transpose()
            .map_err(|e| SolanaMcpError::InvalidSignature(e.to_string()))?;

        let until_sig = until
            .map(|s| s.parse::<Signature>())
            .transpose()
            .map_err(|e| SolanaMcpError::InvalidSignature(e.to_string()))?;

        let config = GetConfirmedSignaturesForAddress2Config {
            commitment: Some(Self::parse_commitment(commitment)),
            limit,
            before: before_sig,
            until: until_sig,
        };

        let response = self
            .client
            .get_signatures_for_address_with_config(&pubkey, config)
            .map_err(SolanaMcpError::from)?;

        Ok(serde_json::to_value(response).unwrap_or(serde_json::json!([])))
    }

    pub fn get_program_accounts(
        &self,
        program_id: &str,
        data_size: Option<usize>,
        memcmp: Option<(usize, &str)>,
        commitment: Option<&str>,
        encoding: Option<&str>,
    ) -> Result<serde_json::Value> {
        // Require at least one filter BEFORE making the RPC call to prevent resource exhaustion
        if data_size.is_none() && memcmp.is_none() {
            return Err(SolanaMcpError::RpcError(
                "get_program_accounts requires at least one filter (data_size or memcmp) to prevent resource exhaustion. \
                 Example: use data_size to filter by account size, or memcmp to filter by specific byte patterns.".to_string()
            ));
        }

        let program = program_id.parse_pubkey()?;

        let mut filters = Vec::new();
        if let Some(size) = data_size {
            filters.push(RpcFilterType::DataSize(size as u64));
        }
        if let Some((offset, bytes)) = memcmp {
            filters.push(RpcFilterType::Memcmp(Memcmp::new(
                offset,
                MemcmpEncodedBytes::Base58(bytes.to_string()),
            )));
        }

        let enc = Self::parse_encoding(encoding.unwrap_or("base64"))?;
        let config = RpcProgramAccountsConfig {
            filters: if filters.is_empty() {
                None
            } else {
                Some(filters)
            },
            account_config: RpcAccountInfoConfig {
                encoding: Some(enc),
                commitment: Some(Self::parse_commitment(commitment)),
                data_slice: None,
                min_context_slot: None,
            },
            with_context: Some(false),
            sort_results: None,
        };

        let accounts = self
            .client
            .get_program_accounts_with_config(&program, config)
            .map_err(SolanaMcpError::from)?;

        let result: Vec<serde_json::Value> = accounts
            .into_iter()
            .map(|(pubkey, account)| {
                serde_json::json!({
                    "pubkey": pubkey.to_string(),
                    "account": Self::serialize_account(account, enc),
                })
            })
            .collect();

        Ok(serde_json::to_value(result).unwrap_or(serde_json::json!([])))
    }

    pub fn simulate_transaction(
        &self,
        transaction: &str,
        encoding: Option<&str>,
        replace_recent_blockhash: bool,
        commitment: Option<&str>,
    ) -> Result<serde_json::Value> {
        if transaction.len() > Self::MAX_TRANSACTION_INPUT_SIZE {
            return Err(SolanaMcpError::RpcError(
                "Transaction input too large".to_string(),
            ));
        }

        let tx_bytes = match encoding.unwrap_or("base64") {
            "base58" => bs58::decode(transaction)
                .into_vec()
                .map_err(|e| SolanaMcpError::RpcError(format!("Failed to decode transaction: {}", e)))?,
            _ => BASE64_STANDARD
                .decode(transaction)
                .map_err(|e| SolanaMcpError::RpcError(format!("Failed to decode transaction: {}", e)))?,
        };

        let tx: Transaction = deserialize(&tx_bytes)
            .map_err(|e| SolanaMcpError::RpcError(format!("Failed to deserialize transaction: {}", e)))?;

        let config = RpcSimulateTransactionConfig {
            commitment: Some(Self::parse_commitment(commitment)),
            encoding: None,
            replace_recent_blockhash,
            sig_verify: false,
            inner_instructions: false,
            accounts: None,
            min_context_slot: None,
        };

        let result = self
            .client
            .simulate_transaction_with_config(&tx, config)
            .map_err(SolanaMcpError::from)?;

        Ok(serde_json::to_value(result.value).unwrap_or(serde_json::json!({"error": "Failed to serialize result"})))
    }

    // ==================== WRITE TOOLS ====================

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
        spawn_blocking(move || {
            client
                .client
                .send_transaction_with_config(&tx, RpcSendTransactionConfig::default())
        })
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
