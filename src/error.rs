use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaMcpError {
    #[error(
        "Invalid address '{0}': expected base58-encoded Solana public key (32-44 characters, e.g., '7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU')"
    )]
    InvalidAddress(String),

    #[error(
        "Invalid signature '{0}': expected base58-encoded Solana signature (87-88 characters from transaction explorer)"
    )]
    InvalidSignature(String),

    #[error(
        "RPC request failed: {0}. Common causes: network timeout, invalid commitment level, or RPC node rate limits. Try again or use a different RPC endpoint."
    )]
    RpcError(String),

    #[error(
        "Invalid RPC endpoint URL '{0}': expected https:// or http:// followed by hostname (e.g., 'https://api.devnet.solana.com')"
    )]
    InvalidEndpoint(String),

    #[error(
        "Invalid encoding '{0}': must be 'base64', 'base58', or 'jsonParsed'. Use 'base64' for raw data, 'jsonParsed' for decoded token accounts."
    )]
    InvalidEncoding(String),

    #[error("Serialization error: {0}. Check that JSON input is valid and matches expected schema.")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal task error: async operation failed to complete. This may indicate a runtime issue.")]
    TaskJoinError,

    #[error("Keypair file not found: '{0}'. Provide a valid path to a Solana keypair JSON file (array of 64 bytes).")]
    KeypairNotFound(String),

    #[error("Invalid keypair: {0}. Expected JSON array of 64 numbers (64-byte Ed25519 keypair).")]
    InvalidKeypair(String),

    #[error(
        "Mainnet operations require --accept-risk flag: {0}. Mainnet involves real funds. Add --accept-risk to proceed."
    )]
    MainnetRiskNotAccepted(String),

    #[error(
        "Transaction failed: {0}. Common causes: insufficient SOL for fees, invalid instruction, or account already exists."
    )]
    TransactionFailed(String),

    #[error(
        "Invalid token account '{0}': expected base58-encoded token account address. For transfers, ensure recipient has an Associated Token Account (ATA)."
    )]
    InvalidTokenAccount(String),
}

pub type Result<T> = std::result::Result<T, SolanaMcpError>;

/// Sanitize RPC error messages to remove potentially sensitive information.
///
/// Uses specific patterns to avoid false positives from common Solana terms
/// like "account", "token" (SPL tokens), or "signature" (transaction signatures)
/// which appear in legitimate error messages.
fn sanitize_rpc_message(msg: &str) -> String {
    // Specific patterns that indicate actual sensitive data
    // These patterns include context to avoid false positives
    let sensitive_patterns = [
        "private key",
        "private_key",
        "privatekey",
        "secret key",
        "secret_key",
        "secretkey",
        "api key",
        "api_key",
        "apikey",
        "bearer ",
        "password:",
        "password=",
        "credential",
        "mnemonic",
        "seed phrase",
        "seedphrase",
    ];

    let msg_lower = msg.to_lowercase();
    for pattern in sensitive_patterns {
        if msg_lower.contains(pattern) {
            return "RPC request failed (details redacted for security)".to_string();
        }
    }

    // Truncate overly long messages that might contain stack traces
    if msg.len() > 200 {
        format!("{}...", &msg[..200])
    } else {
        msg.to_string()
    }
}

impl From<solana_client::client_error::ClientError> for SolanaMcpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        let msg = match err.kind() {
            solana_client::client_error::ClientErrorKind::RpcError(rpc_err) => {
                // Extract just the error kind, not the full error with potentially sensitive details
                let err_msg = format!("{:?}", rpc_err);
                sanitize_rpc_message(&err_msg)
            }
            solana_client::client_error::ClientErrorKind::SerdeJson(_) => "Failed to parse RPC response".to_string(),
            _ => "RPC request failed".to_string(),
        };
        SolanaMcpError::RpcError(msg)
    }
}

#[cfg(test)]
mod tests {
    use solana_client::{client_error::ClientErrorKind, rpc_request::RpcError};

    use super::*;

    #[test]
    fn test_invalid_address_display() {
        let err = SolanaMcpError::InvalidAddress("test".to_string());
        let display_msg = format!("{}", err);
        assert!(
            display_msg.contains("Invalid address"),
            "Expected message to contain 'Invalid address', got: {}",
            display_msg
        );
        assert!(
            display_msg.contains("test"),
            "Expected message to contain 'test', got: {}",
            display_msg
        );
    }

    #[test]
    fn test_invalid_signature_display() {
        let err = SolanaMcpError::InvalidSignature("test".to_string());
        let display_msg = format!("{}", err);
        assert!(
            display_msg.contains("Invalid signature"),
            "Expected message to contain 'Invalid signature', got: {}",
            display_msg
        );
        assert!(
            display_msg.contains("test"),
            "Expected message to contain 'test', got: {}",
            display_msg
        );
    }

    #[test]
    fn test_from_client_error_rpc_error() {
        let rpc_err = RpcError::ForUser("test rpc error".to_string());
        let client_err = solana_client::client_error::ClientError::new_with_request(
            ClientErrorKind::RpcError(rpc_err),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let mcp_err: SolanaMcpError = client_err.into();

        match mcp_err {
            SolanaMcpError::RpcError(msg) => {
                // After sanitization, the message should contain the debug format of the error kind
                // (e.g., "ForUser("test rpc error")") or be redacted if it contains sensitive patterns
                assert!(
                    msg.contains("ForUser") || msg.contains("redacted"),
                    "Expected RpcError message to contain 'ForUser' or 'redacted', got: {}",
                    msg
                );
            }
            _ => panic!("Expected RpcError variant, got: {:?}", mcp_err),
        }
    }

    #[test]
    fn test_sanitize_rpc_message_allows_solana_terms() {
        // These common Solana terms should NOT be redacted
        let test_cases = [
            "Account 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU not found",
            "insufficient account balance",
            "Invalid token account",
            "Token account is frozen",
            "signature verification failed",
            "key not found in account data",
        ];

        for msg in test_cases {
            let sanitized = super::sanitize_rpc_message(msg);
            assert_ne!(
                sanitized, "RPC request failed (details redacted for security)",
                "Message '{}' should NOT be redacted, but it was",
                msg
            );
        }
    }

    #[test]
    fn test_sanitize_rpc_message_redacts_sensitive_data() {
        // These should be redacted
        let test_cases = [
            "error: private key is invalid",
            "secret_key cannot be empty",
            "API_KEY=abc123",
            "bearer token expired",
            "password: admin123",
            "credential verification failed",
            "mnemonic phrase is invalid",
            "seed phrase contains invalid word",
        ];

        for msg in test_cases {
            let sanitized = super::sanitize_rpc_message(msg);
            assert_eq!(
                sanitized, "RPC request failed (details redacted for security)",
                "Message '{}' should be redacted, but got: {}",
                msg, sanitized
            );
        }
    }

    #[test]
    fn test_sanitize_rpc_message_truncates_long_messages() {
        let long_msg = "a".repeat(300);
        let sanitized = super::sanitize_rpc_message(&long_msg);
        assert_eq!(sanitized.len(), 203); // 200 + "..."
        assert!(sanitized.ends_with("..."));
    }

    #[test]
    fn test_sanitize_rpc_message_preserves_short_messages() {
        let short_msg = "short error message";
        let sanitized = super::sanitize_rpc_message(short_msg);
        assert_eq!(sanitized, short_msg);
    }

    #[test]
    fn test_from_client_error_other() {
        let io_err = std::io::Error::other("test io error");
        let client_err = solana_client::client_error::ClientError::new_with_request(
            ClientErrorKind::Io(io_err),
            solana_client::rpc_request::RpcRequest::GetAccountInfo,
        );

        let mcp_err: SolanaMcpError = client_err.into();

        match mcp_err {
            SolanaMcpError::RpcError(msg) => {
                assert_eq!(
                    msg, "RPC request failed",
                    "Expected generic RpcError message, got: {}",
                    msg
                );
            }
            _ => panic!("Expected RpcError variant, got: {:?}", mcp_err),
        }
    }
}
