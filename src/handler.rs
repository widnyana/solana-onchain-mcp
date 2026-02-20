use async_trait::async_trait;
use std::sync::Arc;
use rust_mcp_sdk::schema::{
    CallToolRequestParams, CallToolResult, CallToolError,
    ListToolsResult, PaginatedRequestParams, RpcError,
};
use rust_mcp_sdk::mcp_server::ServerHandler;
use rust_mcp_sdk::McpServer;

use crate::rpc::SolanaRpcClient;
use crate::tools::SolanaTools;

pub struct SolanaMcpHandler {
    client: Arc<SolanaRpcClient>,
}

impl SolanaMcpHandler {
    pub fn new(client: SolanaRpcClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

#[async_trait]
impl ServerHandler for SolanaMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: SolanaTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let client = Arc::clone(&self.client);

        // Convert request parameters into SolanaTools enum
        let tool: SolanaTools = SolanaTools::try_from(params)
            .map_err(CallToolError::new)?;

        // Match the tool variant and execute with client
        match tool {
            SolanaTools::GetBalanceTool(get_balance_tool) => {
                get_balance_tool.call_tool(&client)
            }
            SolanaTools::GetSlotTool(get_slot_tool) => {
                get_slot_tool.call_tool(&client)
            }
            SolanaTools::GetTransactionTool(get_transaction_tool) => {
                get_transaction_tool.call_tool(&client)
            }
        }
    }
}
