use std::sync::Arc;

use async_trait::async_trait;
use rust_mcp_sdk::{
    McpServer,
    mcp_server::ServerHandler,
    schema::{CallToolError, CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError},
};

use crate::{rpc::SolanaRpcClient, tools::SolanaTools};

pub struct SolanaMcpHandler {
    client: SolanaRpcClient,
}

impl SolanaMcpHandler {
    pub fn new(client: SolanaRpcClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ServerHandler for SolanaMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult { tools: SolanaTools::tools(), meta: None, next_cursor: None })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let client = self.client.clone();

        // Convert request parameters into SolanaTools enum
        let tool: SolanaTools = SolanaTools::try_from(params).map_err(CallToolError::new)?;

        // Match the tool variant and execute with client
        match tool {
            SolanaTools::GetBalanceTool(get_balance_tool) => get_balance_tool.call_tool(&client),
            SolanaTools::GetSlotTool(get_slot_tool) => get_slot_tool.call_tool(&client),
            SolanaTools::GetTransactionTool(get_transaction_tool) => get_transaction_tool.call_tool(&client),
        }
    }
}
