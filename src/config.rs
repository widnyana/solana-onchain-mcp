use std::env;

use crate::error::{Result, SolanaMcpError};

const MAINNET_URL: &str = "https://api.mainnet-beta.solana.com";
const DEVNET_URL: &str = "https://api.devnet.solana.com";
const TESTNET_URL: &str = "https://api.testnet.solana.com";

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let network = env::var("SOLANA_NETWORK").unwrap_or_else(|_| "devnet".to_string());

        let rpc_url = match network.as_str() {
            "mainnet" => MAINNET_URL.to_string(),
            "devnet" => DEVNET_URL.to_string(),
            "testnet" => TESTNET_URL.to_string(),
            url => validate_custom_url(url)?,
        };

        Ok(Self { rpc_url })
    }
}

fn validate_custom_url(url: &str) -> Result<String> {
    if !url.starts_with("https://") {
        return Err(SolanaMcpError::InvalidEndpoint(
            "URL must start with https://".to_string(),
        ));
    }

    let parsed = url::Url::parse(url).map_err(|e| SolanaMcpError::InvalidEndpoint(e.to_string()))?;

    if let Some(host) = parsed.host_str()
        && (host == "localhost"
            || host.starts_with("127.")
            || host.starts_with("10.")
            || host.starts_with("192.168.")
            || host.starts_with("172."))
    {
        return Err(SolanaMcpError::InvalidEndpoint(
            "Private network URLs are not allowed".to_string(),
        ));
    }

    Ok(url.to_string())
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env().expect("Invalid default configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_devnet() {
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, DEVNET_URL);
    }

    #[test]
    fn test_mainnet_parsing() {
        unsafe {
            env::set_var("SOLANA_NETWORK", "mainnet");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, MAINNET_URL);
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    fn test_custom_url_rejected_if_not_https() {
        unsafe {
            env::set_var("SOLANA_NETWORK", "http://evil.com");
        }
        let result = Config::from_env();
        assert!(result.is_err());
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    fn test_private_ip_rejected() {
        unsafe {
            env::set_var("SOLANA_NETWORK", "https://127.0.0.1:8899");
        }
        let result = Config::from_env();
        assert!(result.is_err());
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }
}
