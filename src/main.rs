use std::sync::Arc;

use clap::Parser;
use rust_mcp_sdk::{
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    error::SdkResult,
    mcp_server::{McpServerOptions, ServerRuntime, server_runtime},
    schema::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools},
};
use solana_onchain_mcp::{config::Config, handler::SolanaMcpHandler};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "solana-onchain-mcp")]
struct Args {
    /// Accept risk of using private key on mainnet/custom networks
    #[arg(long)]
    accept_risk: bool,
}

#[tokio::main]
async fn main() -> SdkResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let mut config = Config::from_env().expect("Failed to load configuration");

    // CLI flag overrides env var
    if args.accept_risk {
        config.accept_risk = true;
    }

    let handler = SolanaMcpHandler::new(&config).expect("Failed to create handler");

    let server_details = InitializeResult {
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
    };

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
