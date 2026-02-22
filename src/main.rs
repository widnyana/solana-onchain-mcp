use std::sync::Arc;

use clap::Parser;
use rust_mcp_sdk::{
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    error::SdkResult,
    mcp_server::{HyperServerOptions, McpServerOptions, ServerRuntime, hyper_server, server_runtime},
    schema::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools},
};
use solana_onchain_mcp::{config::Config, handler::SolanaMcpHandler};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug, Clone)]
#[command(name = "solana-onchain-mcp")]
struct Args {
    /// Accept risk of using private key on mainnet/custom networks
    #[arg(long)]
    accept_risk: bool,

    /// Enable HTTP transport mode instead of stdio
    #[arg(long)]
    http: bool,

    /// Port for HTTP server (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Host address for HTTP server (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Allow keypair operations in HTTP mode (requires --accept-risk and localhost)
    #[arg(long)]
    http_allow_keypair: bool,
}

fn validate_http_security(args: &Args, config: &mut Config) -> Result<(), String> {
    // http-allow-keypair requires --http
    if args.http_allow_keypair && !args.http {
        return Err("--http-allow-keypair requires --http".into());
    }

    if !args.http {
        return Ok(());
    }
    if !args.http_allow_keypair {
        config.keypair_path = None;
        eprintln!("INFO: HTTP mode running read-only (keypair disabled)");
        return Ok(());
    }
    if !args.accept_risk {
        return Err("--http-allow-keypair requires --accept-risk".into());
    }
    if args.host != "127.0.0.1" && args.host != "localhost" {
        return Err("--http-allow-keypair requires --host 127.0.0.1 for security".into());
    }
    eprintln!("WARN: HTTP mode with signing capability. Ensure reverse proxy with auth!");
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
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Solana blockchain tools. Read: get_balance, get_slot, get_transaction. \
             Write (requires keypair): transfer_sol, transfer_token. \
             Set SOLANA_KEYPAIR_PATH to enable write operations."
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
            ..Default::default()
        },
    );
    eprintln!("HTTP server listening on {}:{}", args.host, args.port);
    server.start().await
}

async fn run_stdio_server(handler: SolanaMcpHandler, server_details: InitializeResult) -> SdkResult<()> {
    let transport = StdioTransport::new(TransportOptions::default())?;
    let server: Arc<ServerRuntime> = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler: handler.to_mcp_server_handler(),
        task_store: None,
        client_task_store: None,
    });
    server.start().await
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let mut config = Config::from_env().expect("Failed to load configuration");

    if args.accept_risk {
        config.accept_risk = true;
    }

    // Security validation for HTTP mode
    if let Err(e) = validate_http_security(&args, &mut config) {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }

    let handler = SolanaMcpHandler::new(&config).expect("Failed to create handler");
    let server_details = build_server_details();

    if args.http {
        run_http_server(handler, &args, server_details).await
    } else {
        run_stdio_server(handler, server_details).await
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_default_args_no_http() {
        let args = Args::try_parse_from(["solana-onchain-mcp"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(!args.http);
        assert_eq!(args.port, 3000);
        assert_eq!(args.host, "127.0.0.1");
        assert!(!args.http_allow_keypair);
    }

    #[test]
    fn test_http_flag_enabled() {
        let args = Args::try_parse_from(["solana-onchain-mcp", "--http"]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.http);
        assert!(!args.http_allow_keypair);
    }

    #[test]
    fn test_custom_port_and_host() {
        let args = Args::try_parse_from([
            "solana-onchain-mcp",
            "--http",
            "--port",
            "8080",
            "--host",
            "0.0.0.0",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.http);
        assert_eq!(args.port, 8080);
        assert_eq!(args.host, "0.0.0.0");
    }

    #[test]
    fn test_http_allow_keypair_flags() {
        let args = Args::try_parse_from([
            "solana-onchain-mcp",
            "--http",
            "--http-allow-keypair",
            "--accept-risk",
        ]);
        assert!(args.is_ok());
        let args = args.unwrap();
        assert!(args.http);
        assert!(args.http_allow_keypair);
        assert!(args.accept_risk);
    }

    #[test]
    fn test_http_allow_keypair_requires_http() {
        let mut config = Config::default();
        let args = Args {
            accept_risk: true,
            http: false, // http not set
            port: 3000,
            host: "127.0.0.1".to_string(),
            http_allow_keypair: true,
        };
        let result = validate_http_security(&args, &mut config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("--http-allow-keypair requires --http"));
    }

    #[test]
    fn test_security_validation_requires_accept_risk() {
        let mut config = Config::default();
        let args = Args {
            accept_risk: false,
            http: true,
            port: 3000,
            host: "127.0.0.1".to_string(),
            http_allow_keypair: true,
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
            http: true,
            port: 3000,
            host: "0.0.0.0".to_string(),
            http_allow_keypair: true,
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
            http: true,
            port: 3000,
            host: "127.0.0.1".to_string(),
            http_allow_keypair: false,
        };
        let result = validate_http_security(&args, &mut config);
        assert!(result.is_ok());
        assert!(config.keypair_path.is_none(), "Keypair should be disabled");
    }
}
