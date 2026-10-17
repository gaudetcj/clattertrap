//! The rmcp adapter: tool names, descriptions, argument schemas, and the conversion of a session's
//! replies into protocol content. All behaviour lives in `session`.

use super::session::{Reply, Session, ToolError};
use base64::Engine;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ErrorData, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// Arguments of `rules`.
#[derive(Deserialize, JsonSchema)]
pub struct RulesArgs {
    /// Return the scene JSON Schema instead of the rules text.
    #[serde(default)]
    schema: bool,
}

/// Arguments of `level`.
#[derive(Deserialize, JsonSchema)]
pub struct LevelArgs {
    /// The level id, as `levels` lists it.
    id: String,
}

/// Arguments of `check`, `look`, and `run`.
#[derive(Deserialize, JsonSchema)]
pub struct SceneArgs {
    /// The scene: `{"level": "<id>", "parts": [...]}`, as the rules describe it.
    scene: Value,
}

/// Arguments of `submit`.
#[derive(Deserialize, JsonSchema)]
pub struct SubmitArgs {
    /// The scene: `{"level": "<id>", "parts": [...]}`, as the rules describe it.
    scene: Value,
    /// A name for the machine, 60 characters or fewer.
    name: String,
    /// One sentence saying what the machine will do, 280 characters or fewer.
    prediction: String,
}

/// The MCP server over one session.
#[derive(Clone)]
pub struct Handler {
    session: Arc<Mutex<Session>>,
    tool_router: ToolRouter<Self>,
}

impl Handler {
    /// A server over `session`.
    #[must_use]
    pub fn new(session: Session) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
            tool_router: Self::tool_router(),
        }
    }

    /// Runs one tool against the session and converts its answer.
    fn answer(
        &self,
        tool: impl FnOnce(&mut Session) -> Result<Reply, ToolError>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| ErrorData::internal_error("the session is poisoned", None))?;
        match tool(&mut session) {
            Ok(reply) => Ok(CallToolResult::success(blocks(reply))),
            Err(ToolError::Refused(text)) => {
                Ok(CallToolResult::error(vec![ContentBlock::text(text)]))
            }
            Err(ToolError::Internal(e)) => Err(ErrorData::internal_error(format!("{e:#}"), None)),
        }
    }
}

/// A reply as content blocks: the text, then the picture when there is one.
fn blocks(reply: Reply) -> Vec<ContentBlock> {
    let mut out = vec![ContentBlock::text(reply.text)];
    if let Some(png) = reply.png {
        let data = base64::engine::general_purpose::STANDARD.encode(png);
        out.push(ContentBlock::image(data, "image/png"));
    }
    out
}

#[tool_router]
impl Handler {
    #[tool(
        description = "The rules every contestant plays by: units, materials, every part kind with its fields and sizes, the goal objects, and the placement rules. Pass schema true for the scene JSON Schema instead."
    )]
    fn rules(&self, Parameters(args): Parameters<RulesArgs>) -> Result<CallToolResult, ErrorData> {
        self.answer(|_| Session::rules(args.schema))
    }

    #[tool(description = "Every level with its tier and goal kind, one per line.")]
    fn levels(&self) -> Result<CallToolResult, ErrorData> {
        self.answer(|_| Session::levels())
    }

    #[tool(
        description = "One level in words and JSON, with a picture of it before anything is placed. In a pilot session only your assigned level."
    )]
    fn level(&self, Parameters(args): Parameters<LevelArgs>) -> Result<CallToolResult, ErrorData> {
        self.answer(|s| s.level(&args.id))
    }

    #[tool(
        description = "Validate a scene against its level without running it. Answers ok with the part count, or every problem with its JSON path, the rule, and a fix."
    )]
    fn check(&self, Parameters(args): Parameters<SceneArgs>) -> Result<CallToolResult, ErrorData> {
        self.answer(|s| s.check(&args.scene))
    }

    #[tool(
        description = "A picture of a valid scene at the start of its run, before anything moves. Lists the problems instead when the rules reject the scene."
    )]
    fn look(&self, Parameters(args): Parameters<SceneArgs>) -> Result<CallToolResult, ErrorData> {
        self.answer(|s| s.look(&args.scene))
    }

    #[tool(
        description = "Simulate a valid scene to its end: every event with its time, the outcome, how many runs you have left, and a picture of the final frame. Costs one run from your budget; a rejected scene costs nothing."
    )]
    fn run(&self, Parameters(args): Parameters<SceneArgs>) -> Result<CallToolResult, ErrorData> {
        self.answer(|s| s.run(&args.scene))
    }

    #[tool(
        description = "Submit your machine, once: the scene, a machine name of 60 characters or fewer, and a one-sentence prediction of 280 or fewer. The scene is run and its outcome is final; nothing more can be done afterwards."
    )]
    fn submit(
        &self,
        Parameters(args): Parameters<SubmitArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.answer(|s| s.submit(&args.scene, &args.name, &args.prediction))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Handler {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Clattertrap: build a Rube Goldberg machine that meets the level's goal. Read rules and level first, check scenes before you run or submit them, and submit exactly once.",
        )
    }
}
