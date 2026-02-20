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

fn is_private_ip(host: &str) -> bool {
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        match ip {
            std::net::IpAddr::V4(v4) => {
                v4.is_loopback() || v4.is_private() || v4.is_link_local()
            }
            std::net::IpAddr::V6(v6) => {
                v6.is_loopback() || v6.is_unique_local()
            }
        }
    } else {
        host == "localhost"
    }
}

fn validate_custom_url(url: &str) -> Result<String> {
    // Allow both http:// and https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(SolanaMcpError::InvalidEndpoint(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    let parsed = url::Url::parse(url).map_err(|e| SolanaMcpError::InvalidEndpoint(e.to_string()))?;

    // Only block private IPs for https:// (http:// allowed for local dev)
    if url.starts_with("https://") {
        if let Some(host) = parsed.host_str() {
            if is_private_ip(host) {
                return Err(SolanaMcpError::InvalidEndpoint(
                    "Private network URLs are not allowed for https://".to_string(),
                ));
            }
        }
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_default_is_devnet() {
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, DEVNET_URL);
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_custom_url_requires_http_or_https() {
        // Invalid scheme should be rejected (ftp is not allowed)
        unsafe {
            env::set_var("SOLANA_NETWORK", "ftp://evil.com");
        }
        let result = Config::from_env();
        assert!(result.is_err(), "ftp:// should be rejected");
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    #[serial]
    fn test_https_url_accepted() {
        // https:// with public host should be accepted
        unsafe {
            env::set_var("SOLANA_NETWORK", "https://api.example.com");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, "https://api.example.com");
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    #[serial]
    fn test_https_private_ip_rejected() {
        // Private IPs are blocked for https://
        unsafe {
            env::set_var("SOLANA_NETWORK", "https://127.0.0.1:8899");
        }
        let result = Config::from_env();
        assert!(result.is_err());
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    #[serial]
    fn test_http_allowed() {
        // http:// URLs should be allowed for public hosts
        unsafe {
            env::set_var("SOLANA_NETWORK", "http://example.com:8899");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, "http://example.com:8899");
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    #[serial]
    fn test_http_private_allowed() {
        // http:// with private IP should be allowed for local dev
        unsafe {
            env::set_var("SOLANA_NETWORK", "http://127.0.0.1:8899");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.rpc_url, "http://127.0.0.1:8899", "http:// with private IP should be allowed");
        unsafe {
            env::remove_var("SOLANA_NETWORK");
        }
    }

    #[test]
    fn test_is_private_ip_loopback() {
        assert!(is_private_ip("127.0.0.1"));
        assert!(is_private_ip("localhost"));
    }

    #[test]
    fn test_is_private_ip_private_ranges() {
        // 10.0.0.0/8
        assert!(is_private_ip("10.0.0.1"));
        assert!(is_private_ip("10.255.255.255"));
        // 172.16.0.0/12
        assert!(is_private_ip("172.16.0.1"));
        assert!(is_private_ip("172.31.255.255"));
        // 192.168.0.0/16
        assert!(is_private_ip("192.168.0.1"));
        assert!(is_private_ip("192.168.255.255"));
    }

    #[test]
    fn test_is_private_ip_link_local() {
        // 169.254.0.0/16
        assert!(is_private_ip("169.254.0.1"));
        assert!(is_private_ip("169.254.255.255"));
    }

    #[test]
    fn test_is_private_ip_public_not_blocked() {
        // 172.15.x.x is public (outside 172.16.0.0/12)
        assert!(!is_private_ip("172.15.0.1"));
        // 172.32.x.x is public (outside 172.16.0.0/12)
        assert!(!is_private_ip("172.32.0.1"));
        // Regular public IPs
        assert!(!is_private_ip("8.8.8.8"));
        assert!(!is_private_ip("1.1.1.1"));
    }

    #[test]
    fn test_is_private_ip_ipv6() {
        // IPv6 loopback
        assert!(is_private_ip("::1"));
        // IPv6 unique local (fc00::/7)
        assert!(is_private_ip("fc00::1"));
        assert!(is_private_ip("fdff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"));
        // IPv6 public
        assert!(!is_private_ip("2001:4860:4860::8888"));
    }
}
