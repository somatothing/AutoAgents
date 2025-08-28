#[cfg(feature = "mcp")]
use crate::tool::{ToolCallError, ToolRuntime, ToolT};
#[cfg(feature = "mcp")]
use serde_json::Value;

#[cfg(feature = "mcp")]
#[derive(Debug)]
pub struct McpTool;

#[cfg(feature = "mcp")]
impl ToolT for McpTool {
    fn name(&self) -> &'static str { "mcp_call" }
    fn description(&self) -> &'static str { "Call an MCP server method (stub)." }
    fn args_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {"type": "string"},
                "method": {"type": "string"},
                "params": {"type": "object"}
            },
            "required": ["server", "method"]
        })
    }
}

#[cfg(feature = "mcp")]
impl ToolRuntime for McpTool {
    fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let server = args.get("server").and_then(|v| v.as_str()).unwrap_or("");
        let method = args.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = args.get("params").cloned().unwrap_or(serde_json::json!({}));
        Ok(serde_json::json!({
            "server": server,
            "method": method,
            "echo": params,
            "note": "MCP integration is stubbed. Provide a real client to enable remote calls."
        }))
    }
}

