//! The MCP client the endpoint provider drives: the same `clattertrap mcp` child Claude Code gets,
//! listed as tool definitions and called one at a time.

use super::chat::ToolDef;
use crate::records::transcript::Content;
use anyhow::{Context, Result};
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, ContentBlock};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::TokioChildProcess;
use serde_json::{Map, Value};
use std::fs::File;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// A connected MCP server.
pub struct Server {
    service: RunningService<RoleClient, ()>,
}

/// What a tool answered.
#[derive(Debug, Default)]
pub struct Answer {
    /// The blocks as the transcript keeps them.
    pub content: Vec<Content>,
    /// The text blocks joined, for the tool message.
    pub text: String,
    /// Every PNG, base64, for the picture message.
    pub pngs: Vec<String>,
    /// Whether the tool refused the call.
    pub is_error: bool,
}

impl Server {
    /// Spawns `exe args` in `cwd`, sending its stderr to `stderr`, and completes the handshake.
    ///
    /// # Errors
    /// When the process cannot start or the handshake fails.
    pub async fn start(exe: &Path, args: &[String], cwd: &Path, stderr: File) -> Result<Self> {
        let mut cmd = Command::new(exe);
        cmd.args(args).current_dir(cwd);
        let (transport, _) = TokioChildProcess::builder(cmd)
            .stderr(Stdio::from(stderr))
            .spawn()
            .context("starting the MCP server")?;
        let service = ().serve(transport).await.context("the MCP handshake failed")?;
        Ok(Self { service })
    }

    /// The server's instructions, which become the system message.
    #[must_use]
    pub fn instructions(&self) -> Option<String> {
        self.service
            .peer_info()
            .and_then(|info| info.instructions.clone())
    }

    /// Every tool the server lists, sorted by name.
    ///
    /// # Errors
    /// When the listing fails.
    pub async fn tools(&self) -> Result<Vec<ToolDef>> {
        let mut tools = self
            .service
            .list_all_tools()
            .await
            .context("listing the tools")?;
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tools
            .into_iter()
            .map(|t| ToolDef {
                name: t.name.into_owned(),
                description: t.description.map(Into::into).unwrap_or_default(),
                parameters: Value::Object((*t.input_schema).clone()),
            })
            .collect())
    }

    /// Calls one tool. Text and picture blocks are kept; any other kind is dropped.
    ///
    /// # Errors
    /// When the call cannot be made; a refusal is an answer, not an error.
    pub async fn call(&self, name: &str, arguments: Map<String, Value>) -> Result<Answer> {
        let params = CallToolRequestParams::new(name.to_owned()).with_arguments(arguments);
        let result = self
            .service
            .call_tool(params)
            .await
            .with_context(|| format!("calling {name}"))?;
        let mut answer = Answer {
            is_error: result.is_error == Some(true),
            ..Answer::default()
        };
        for block in result.content {
            match block {
                ContentBlock::Text(t) => {
                    if !answer.text.is_empty() {
                        answer.text.push('\n');
                    }
                    answer.text.push_str(&t.text);
                    answer.content.push(Content::Text {
                        text: t.text.clone(),
                    });
                }
                ContentBlock::Image(i) => {
                    answer.pngs.push(i.data.clone());
                    answer.content.push(Content::Image { omitted: true });
                }
                _ => {}
            }
        }
        Ok(answer)
    }

    /// Stops the server.
    pub async fn close(self) {
        let _ = self.service.cancel().await;
    }
}
