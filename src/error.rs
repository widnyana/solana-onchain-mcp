use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaMcpError {
    #[error("Invalid address '{0}': must be a valid Solana base58 public key (32-44 characters)")]
    InvalidAddress(String),

    #[error("Invalid signature '{0}': must be a valid Solana base58 signature (87-88 characters)")]
    InvalidSignature(String),

    #[error("RPC request failed: {0}")]
    RpcError(String),

    #[error("Invalid RPC endpoint URL: {0}")]
    InvalidEndpoint(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal task error")]
    TaskJoinError,

    #[error("Keypair file not found: {0}")]
    KeypairNotFound(String),

    #[error("Invalid keypair: {0}")]
    InvalidKeypair(String),

    #[error("Mainnet requires --accept-risk: {0}")]
    MainnetRiskNotAccepted(String),

    #[error("Transaction failed: {0}")]
    #[allow(dead_code)]
    TransactionFailed(String),

    #[error("Invalid token account: {0}")]
    #[allow(dead_code)]
    InvalidTokenAccount(String),
}

pub type Result<T> = std::result::Result<T, SolanaMcpError>;

impl From<solana_client::client_error::ClientError> for SolanaMcpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        let msg = match err.kind() {
            solana_client::client_error::ClientErrorKind::RpcError(rpc_err) => {
                format!("RPC error: {}", rpc_err)
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
                assert!(
                    msg.contains("RPC error"),
                    "Expected RpcError message to contain 'RPC error', got: {}",
                    msg
                );
            }
            _ => panic!("Expected RpcError variant, got: {:?}", mcp_err),
        }
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
