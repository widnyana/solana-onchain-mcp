use std::path::PathBuf;

use clap::Parser;
use rust_mcp_sdk::{
    ToMcpServerHandler,
    error::SdkResult,
    mcp_server::{HyperServerOptions, hyper_server},
    schema::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools},
};
use serde_json::Map;
use solana_onchain_mcp::{config::Config, handler::SolanaMcpHandler};
mod logging;

use tracing::{error, info, warn};

#[derive(Parser, Debug, Clone)]
#[command(name = "solana-onchain-mcp")]
struct Args {
    /// Accept risk of using private key on mainnet/custom networks
    #[arg(long)]
    accept_risk: bool,

    /// Port for HTTP server (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Host address for HTTP server (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Allow keypair operations in HTTP mode (requires --accept-risk and localhost)
    #[arg(long)]
    http_allow_keypair: bool,

    #[arg(
        long,
        value_name = "LEVEL",
        help = "Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE [env: LOG_LEVEL=]"
    )]
    log_level: Option<String>,

    #[arg(
        long,
        value_name = "FORMAT",
        help = "Log format: pretty, json [env: LOG_FORMAT=]"
    )]
    log_format: Option<String>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Write logs to file [env: LOG_PATH=]"
    )]
    log_path: Option<PathBuf>,
}

fn validate_http_security(args: &Args, config: &mut Config) -> Result<(), String> {
    if !args.http_allow_keypair {
        config.keypair_path = None;
        info!("HTTP mode running read-only (keypair disabled)");
        return Ok(());
    }
    if !args.accept_risk {
        return Err("--http-allow-keypair requires --accept-risk".into());
    }
    if args.host != "127.0.0.1" && args.host != "localhost" {
        return Err("--http-allow-keypair requires --host 127.0.0.1 for security".into());
    }
    warn!("HTTP mode with signing capability enabled. Only use on localhost!");
    Ok(())
}

fn build_server_details() -> InitializeResult {
    InitializeResult {
        server_info: Implementation {
            name: "solana-onchain-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Solana On-Chain MCP Server".to_string()),
            description: Some("MCP server for Solana blockchain operations".to_string()),
            icons: vec![],
            website_url: Some("https://github.com/widnyana/solana-onchain-mcp".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            resources: None,
            completions: Some(Map::new()),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Solana blockchain tools.\n\
             Read (no keypair): get_balance, get_account_info, get_multiple_accounts, \
             get_token_accounts_by_owner, get_program_accounts, get_transaction, \
             get_signatures_for_address, get_signature_status, get_slot, \
             simulate_transaction, get_server_info, query_transactions, \
             inspect_transaction_raw, inspect_transaction_humanized.\n\
             Write (requires SOLANA_KEYPAIR_PATH): transfer_sol, transfer_token, \
             create_associated_token_account, approve_token, revoke_token, close_token_account."
                .to_string(),
        ),
        protocol_version: ProtocolVersion::V2025_11_25.into(),
    }
}

async fn run_http_server(handler: SolanaMcpHandler, args: &Args, server_details: InitializeResult) -> SdkResult<()> {
    let server = hyper_server::create_server(
        server_details,
        handler.to_mcp_server_handler(),
        HyperServerOptions {
            host: args.host.clone(),
            port: args.port,
            sse_support: true,
            message_observer: None,
            ..Default::default()
        },
    );
    info!(host = %args.host, port = args.port, "HTTP server listening");
    server.start().await
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    let args = Args::parse();

    let mut logging_config = logging::LoggingConfig::from_env();
    logging_config.apply_cli_overrides(
        args.log_level.clone(),
        args.log_format.clone(),
        args.log_path.clone(),
    );
    if let Err(e) = logging_config.init() {
        eprintln!("Warning: logging initialization failed: {}", e);
    }

    let mut config = Config::from_env().expect("Failed to load configuration");

    if args.accept_risk {
        config.accept_risk = true;
    }
    if args.accept_risk {
        warn!("CLI flag --accept-risk is set. Write operations enabled on mainnet/custom networks.");
    }

    if let Err(e) = validate_http_security(&args, &mut config) {
        error!(error = %e, "Security validation failed");
        std::process::exit(1);
    }

    if config.is_mainnet_or_custom() {
        warn!("SERVER STARTING ON MAINNET/CUSTOM NETWORK - Write operations will use real assets.");
    }

    info!(
        network = ?config.network_type,
        keypair = if config.keypair_path.is_some() { "enabled" } else { "read-only" },
        "solana-onchain-mcp v{} ready",
        env!("CARGO_PKG_VERSION")
    );

    let handler = SolanaMcpHandler::new(&config).expect("Failed to create handler");
    let server_details = build_server_details();

    run_http_server(handler, &args, server_details).await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_default_args() {
        let args = Args::try_parse_from(["solana-onchain-mcp"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.port, 3000);
        assert_eq!(args.host, "127.0.0.1");
        assert!(!args.http_allow_keypair);
    }

    #[test]
    fn test_custom_port_and_host() {
        let args = Args::try_parse_from(["solana-onchain-mcp", "--port", "8080", "--host", "0.0.0.0"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "0.0.0.0");
    }

    #[test]
    fn test_http_allow_keypair_flags() {
        let args = Args::try_parse_from([
            "solana-onchain-mcp",
            "--http-allow-keypair",
            "--accept-risk",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.http_allow_keypair);
        assert!(args.accept_risk);
    }

    #[test]
    fn test_security_validation_requires_accept_risk() {
        let mut config = Config::default();
        let args = Args {
            accept_risk: false,
            port: 3000,
            host: "127.0.0.1".to_string(),
            http_allow_keypair: true,
            log_level: None,
            log_format: None,
            log_path: None,
        };
        let result = validate_http_security(&args, &mut config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--accept-risk"));
    }

    #[test]
    fn test_security_validation_requires_localhost() {
        let mut config = Config::default();
        let args = Args {
            accept_risk: true,
            port: 3000,
            host: "0.0.0.0".to_string(),
            http_allow_keypair: true,
            log_level: None,
            log_format: None,
            log_path: None,
        };
        let result = validate_http_security(&args, &mut config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("127.0.0.1"));
    }

    #[test]
    fn test_security_disables_keypair_by_default() {
        let mut config = Config {
            keypair_path: Some(PathBuf::from("/some/key.json")),
            ..Default::default()
        };
        let args = Args {
            accept_risk: false,
            port: 3000,
            host: "127.0.0.1".to_string(),
            http_allow_keypair: false,
            log_level: None,
            log_format: None,
            log_path: None,
        };
        let result = validate_http_security(&args, &mut config);
        assert!(result.is_ok());
        assert!(config.keypair_path.is_none(), "Keypair should be disabled");
    }
}
