//! The contestant interface: an MCP server over stdio with the tools `rules`, `levels`, `level`,
//! `check`, `look`, `run`, and `submit`.

pub mod arm;
mod handler;
pub mod session;

#[cfg(test)]
mod tests;

/// The contestant tools, as the server lists them.
pub const TOOL_NAMES: [&str; 7] = ["rules", "levels", "level", "check", "look", "run", "submit"];

use crate::catalog;
use anyhow::{Context, Result};
use arm::Arm;
use rmcp::ServiceExt;
use std::path::PathBuf;

/// Serves one session over stdin and stdout until the client disconnects.
///
/// # Errors
/// When the level id is unknown, the runtime cannot start, or the protocol handshake fails.
pub fn serve(level: Option<&str>, arm: Arm, out: Option<PathBuf>) -> Result<()> {
    let level = level
        .map(|id| catalog::level(catalog::find(id)?))
        .transpose()?;
    let handler = handler::Handler::new(session::Session::new(level, arm, out));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("starting the runtime")?;
    runtime.block_on(async {
        let service = handler
            .serve(rmcp::transport::stdio())
            .await
            .context("the MCP handshake failed")?;
        service.waiting().await.context("the server task failed")?;
        Ok(())
    })
}
