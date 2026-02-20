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
    #[allow(dead_code)]
    TaskJoinError,
}

pub type Result<T> = std::result::Result<T, SolanaMcpError>;

impl From<solana_client::client_error::ClientError> for SolanaMcpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        let msg = match err.kind() {
            solana_client::client_error::ClientErrorKind::RpcError(rpc_err) => {
                format!("RPC error: {}", rpc_err)
            }
            solana_client::client_error::ClientErrorKind::SerdeJson(_) => {
                "Failed to parse RPC response".to_string()
            }
            _ => "RPC request failed".to_string(),
        };
        SolanaMcpError::RpcError(msg)
    }
}
