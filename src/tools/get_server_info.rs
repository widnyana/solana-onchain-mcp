use rust_mcp_sdk::{
    macros::{JsonSchema, mcp_tool},
    schema::{CallToolError, CallToolResult},
};

use crate::{
    config::{Config, NetworkType},
    keypair::LoadedKeypair,
    tools::json_result,
};

#[mcp_tool(
    name = "get_server_info",
    description = "Get MCP server configuration and capabilities. Returns network info, write capability, and wallet address if configured. Use this to discover what network you're on and whether write operations are available."
)]
#[derive(Debug, Default, serde::Deserialize, serde::Serialize, JsonSchema)]
pub struct GetServerInfoTool {}

impl GetServerInfoTool {
    pub fn call_tool(&self, config: &Config, keypair: Option<&LoadedKeypair>) -> Result<CallToolResult, CallToolError> {
        Ok(json_result(
            serde_json::json!({
                "network": match config.network_type {
                    NetworkType::Mainnet => "mainnet",
                    NetworkType::Devnet => "devnet",
                    NetworkType::Testnet => "testnet",
                    NetworkType::Custom(ref url) => url.as_str(),
                },
                "rpc_url": &config.rpc_url,
                "write_enabled": keypair.is_some(),
                "wallet_address": keypair.as_ref().map(|k| k.pubkey.as_str()),
            }),
            "Server info unavailable",
        ))
    }
}
