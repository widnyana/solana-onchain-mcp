use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaMcpError {
    // Input validation
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    // Solana errors - boxed to reduce enum size
    #[error("RPC error: {0}")]
    RpcError(#[from] Box<solana_client::client_error::ClientError>),

    #[error("Invalid RPC endpoint URL: {0}")]
    InvalidEndpoint(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl From<solana_client::client_error::ClientError> for SolanaMcpError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        SolanaMcpError::RpcError(Box::new(err))
    }
}

pub type Result<T> = std::result::Result<T, SolanaMcpError>;
