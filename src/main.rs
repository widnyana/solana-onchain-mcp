// Allow enum variant naming pattern from macro-generated code
#![allow(clippy::enum_variant_names)]

mod config;
mod error;
mod handler;
mod rpc;
mod tools;

use std::sync::Arc;

use config::Config;
use handler::SolanaMcpHandler;
use rpc::SolanaRpcClient;
use rust_mcp_sdk::{
    McpServer, StdioTransport, ToMcpServerHandler, TransportOptions,
    error::SdkResult,
    mcp_server::{McpServerOptions, ServerRuntime, server_runtime},
    schema::{Implementation, InitializeResult, ProtocolVersion, ServerCapabilities, ServerCapabilitiesTools},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> SdkResult<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env().expect("Failed to load configuration");
    let client = SolanaRpcClient::new(&config);
    let handler = SolanaMcpHandler::new(client);

    let server_details = InitializeResult {
        server_info: Implementation {
            name: "solana-onchain-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Solana On-Chain MCP Server".to_string()),
            description: Some("MCP server for querying Solana blockchain data".to_string()),
            icons: vec![],
            website_url: Some("https://github.com/solana-labs/solana".to_string()),
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Solana blockchain query tools. Use get_balance to check SOL balance, \
             get_slot to get current block height, and get_transaction to fetch transaction details."
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
