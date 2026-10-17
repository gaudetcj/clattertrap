//! An MCP client over `clattertrap mcp` running as a child process.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use base64::Engine;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::TokioChildProcess;
use serde_json::Value;
use std::fmt::Write;
use tokio::process::Command;

/// A connected client.
pub struct Client {
    service: RunningService<RoleClient, ()>,
}

impl Client {
    /// Starts `clattertrap mcp <args>` from the repository root and connects to it.
    pub async fn start(args: &[&str]) -> Self {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_clattertrap"));
        cmd.arg("mcp").args(args).current_dir(super::repo_root());
        let transport = TokioChildProcess::new(cmd).unwrap();
        let service = ().serve(transport).await.unwrap();
        Self { service }
    }

    /// Every tool the server lists, sorted by name.
    pub async fn tools(&self) -> Vec<rmcp::model::Tool> {
        let mut tools = self.service.list_all_tools().await.unwrap();
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        tools
    }

    /// Calls one tool with JSON arguments.
    pub async fn call(&self, name: &'static str, args: Value) -> CallToolResult {
        let params = match args {
            Value::Object(map) => CallToolRequestParams::new(name).with_arguments(map),
            _ => CallToolRequestParams::new(name),
        };
        self.service.call_tool(params).await.unwrap()
    }

    /// Calls a tool and renders its answer.
    pub async fn text(&self, name: &'static str, args: Value) -> String {
        render(&self.call(name, args).await)
    }

    /// Stops the server.
    pub async fn close(self) {
        self.service.cancel().await.unwrap();
    }
}

/// A tool result as text: text blocks as they are, an image as `<png N bytes>`, and `error:` first
/// when the server marked it as one.
pub fn render(result: &CallToolResult) -> String {
    let mut out = String::new();
    if result.is_error == Some(true) {
        out.push_str("error:\n");
    }
    for block in &result.content {
        match block {
            ContentBlock::Text(t) => out.push_str(&t.text),
            ContentBlock::Image(i) => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(&i.data)
                    .unwrap();
                assert_eq!(i.mime_type, "image/png");
                assert!(bytes.starts_with(b"\x89PNG"), "image is not a PNG");
                write!(out, "<png {} bytes>", bytes.len()).unwrap();
            }
            other => panic!("unexpected content block {other:?}"),
        }
        out.push('\n');
    }
    out
}
