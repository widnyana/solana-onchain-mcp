use std::sync::Arc;

use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{
        CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError,
        TextContent,
    },
};

use crate::{config::Config, error::SolanaMcpError, keypair::LoadedKeypair, rpc::SolanaRpcClient, tools::SolanaTools};

pub struct SolanaMcpHandler {
    client: SolanaRpcClient,
    keypair: Option<LoadedKeypair>,
}

impl SolanaMcpHandler {
    fn require_keypair(&self) -> Result<&LoadedKeypair, CallToolError> {
        self.keypair.as_ref().ok_or_else(|| {
            CallToolError::new(SolanaMcpError::InvalidKeypair(
                "No keypair configured. Set SOLANA_KEYPAIR_PATH environment variable.".to_string(),
            ))
        })
    }

    pub fn new(config: &Config) -> Result<Self, SolanaMcpError> {
        let client = SolanaRpcClient::new(config);

        let keypair = match &config.keypair_path {
            None => None,
            Some(path) => {
                match crate::keypair::load_keypair(path) {
                    Ok(kp) => {
                        // Security guard: mainnet/custom requires accept_risk
                        if config.is_mainnet_or_custom() && !config.accept_risk {
                            return Err(SolanaMcpError::MainnetRiskNotAccepted(
                                "Add --accept-risk or SOLANA_ACCEPT_RISK=true to enable write operations on mainnet/custom networks".to_string()
                            ));
                        }
                        Some(kp)
                    }
                    Err(e) => {
                        eprintln!("WARN: {}. Continuing in read-only mode.", e);
                        None
                    }
                }
            }
        };

        Ok(Self { client, keypair })
    }
}

#[async_trait]
impl ServerHandler for SolanaMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        // Only list write tools if keypair is configured
        let tools = if self.keypair.is_some() {
            SolanaTools::tools()
        } else {
            // Filter out write tools when no keypair
            SolanaTools::tools()
                .into_iter()
                .filter(|t| !matches!(t.name.as_str(), "transfer_sol" | "transfer_token"))
                .collect()
        };
        Ok(ListToolsResult { tools, meta: None, next_cursor: None })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let client = self.client.clone();

        // Convert request parameters into SolanaTools enum
        let tool: SolanaTools = SolanaTools::try_from(params).map_err(CallToolError::new)?;

        // Match the tool variant and execute
        match tool {
            SolanaTools::GetAccountInfoTool(get_account_info_tool) => get_account_info_tool.call_tool(&client),
            SolanaTools::GetBalanceTool(get_balance_tool) => get_balance_tool.call_tool(&client),
            SolanaTools::GetMultipleAccountsTool(get_multiple_accounts_tool) => {
                get_multiple_accounts_tool.call_tool(&client)
            }
            SolanaTools::GetProgramAccountsTool(get_program_accounts_tool) => {
                get_program_accounts_tool.call_tool(&client)
            }
            SolanaTools::GetSignaturesForAddressTool(get_signatures_for_address_tool) => {
                get_signatures_for_address_tool.call_tool(&client)
            }
            SolanaTools::GetSlotTool(get_slot_tool) => get_slot_tool.call_tool(&client),
            SolanaTools::GetTokenAccountsByOwnerTool(get_token_accounts_by_owner_tool) => {
                get_token_accounts_by_owner_tool.call_tool(&client)
            }
            SolanaTools::GetTransactionTool(get_transaction_tool) => get_transaction_tool.call_tool(&client),
            SolanaTools::SimulateTransactionTool(simulate_transaction_tool) => {
                simulate_transaction_tool.call_tool(&client)
            }
            SolanaTools::TransferSolTool(transfer_sol_tool) => {
                let keypair = self.require_keypair()?;

                let result = transfer_sol_tool
                    .call_tool(&client, keypair)
                    .await
                    .map_err(CallToolError::new)?;

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Transfer successful".to_string()),
                )]))
            }
            SolanaTools::TransferTokenTool(transfer_token_tool) => {
                let keypair = self.require_keypair()?;

                let result = transfer_token_tool
                    .call_tool(&client, keypair)
                    .await
                    .map_err(CallToolError::new)?;

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Transfer successful".to_string()),
                )]))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use solana_sdk::signature::Keypair;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::config::NetworkType;

    fn create_valid_keypair_file() -> NamedTempFile {
        let keypair = Keypair::new();
        let keypair_bytes: Vec<u8> = keypair.to_bytes().to_vec();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(
            temp_file,
            "{}",
            serde_json::to_string(&keypair_bytes).unwrap()
        )
        .unwrap();
        temp_file
    }

    #[test]
    fn test_handler_devnet_with_keypair_succeeds() {
        let keypair_file = create_valid_keypair_file();

        let config = Config {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            network_type: NetworkType::Devnet,
            keypair_path: Some(keypair_file.path().to_path_buf()),
            accept_risk: false,
        };

        let result = SolanaMcpHandler::new(&config);
        assert!(
            result.is_ok(),
            "Handler creation should succeed for devnet with keypair"
        );

        let handler = result.unwrap();
        assert!(handler.keypair.is_some(), "Keypair should be loaded");
    }

    #[test]
    fn test_handler_mainnet_without_accept_risk_fails() {
        let keypair_file = create_valid_keypair_file();

        let config = Config {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            network_type: NetworkType::Mainnet,
            keypair_path: Some(keypair_file.path().to_path_buf()),
            accept_risk: false,
        };

        let result = SolanaMcpHandler::new(&config);
        assert!(
            result.is_err(),
            "Handler creation should fail for mainnet without accept_risk"
        );

        match result {
            Err(SolanaMcpError::MainnetRiskNotAccepted(_)) => {}
            _ => panic!("Expected MainnetRiskNotAccepted error"),
        }
    }

    #[test]
    fn test_handler_mainnet_with_accept_risk_succeeds() {
        let keypair_file = create_valid_keypair_file();

        let config = Config {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            network_type: NetworkType::Mainnet,
            keypair_path: Some(keypair_file.path().to_path_buf()),
            accept_risk: true,
        };

        let result = SolanaMcpHandler::new(&config);
        assert!(
            result.is_ok(),
            "Handler creation should succeed for mainnet with accept_risk=true"
        );

        let handler = result.unwrap();
        assert!(handler.keypair.is_some(), "Keypair should be loaded");
    }

    #[test]
    fn test_handler_invalid_keypair_continues_read_only() {
        let config = Config {
            rpc_url: "https://api.devnet.solana.com".to_string(),
            network_type: NetworkType::Devnet,
            keypair_path: Some(std::path::PathBuf::from("/nonexistent/keypair.json")),
            accept_risk: false,
        };

        let result = SolanaMcpHandler::new(&config);
        assert!(
            result.is_ok(),
            "Handler creation should succeed even with invalid keypair path"
        );

        let handler = result.unwrap();
        assert!(
            handler.keypair.is_none(),
            "Keypair should be None when keypair file is invalid"
        );
    }
}
