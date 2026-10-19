//! The chat endpoint provider: an OpenAI-compatible chat completions API, hosted or a
//! local llama.cpp server, driven through the same MCP server Claude Code gets. Pictures go to the
//! model as `image_url` parts, so the endpoint must accept them.

use super::chat::{
    Reply, ToolCall, function_tools, image_message, parse_reply, request, retry_after, tool_message,
};
use super::provider::{Finished, Provider, Session};
use super::server_args;
use super::status::is_ours;
use super::tools::Server;
use crate::records::transcript::{Content, Record};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::fs::File;
use std::time::Instant;

/// A chat endpoint and the model to ask for.
#[derive(Debug)]
pub struct Endpoint {
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl Endpoint {
    /// A provider posting to `base_url/chat/completions` for `model`, with `api_key` as a bearer
    /// token when there is one.
    #[must_use]
    pub fn new(base_url: &str, api_key: Option<String>, model: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key,
            model: model.to_owned(),
        }
    }
}

impl Provider for Endpoint {
    async fn run(&self, s: &Session<'_>) -> Result<Finished> {
        let started = Instant::now();
        let stderr = File::create(s.dir.join("stderr.txt")).context("creating stderr.txt")?;
        let args = server_args(s.level, s.arm, s.dir);
        let server = match Server::start(s.exe, &args, s.dir, stderr).await {
            Ok(server) => server,
            Err(e) => {
                let end = End::failed("error_mcp", format!("{e:#}"));
                let mut finished = finished(vec![], None, 0, started, String::new(), end);
                finished.connected = false;
                return Ok(finished);
            }
        };
        let mut chat = Chat {
            endpoint: self,
            server: &server,
            client: reqwest::Client::new(),
            messages: vec![
                json!({ "role": "system", "content": server.instructions().unwrap_or_default() }),
                json!({ "role": "user", "content": s.prompt }),
            ],
            tools: function_tools(&server.tools().await?),
            records: vec![],
            raw: String::new(),
            cost: None,
            turns: 0,
            deadline: started + s.wall_clock,
            cap: s.max_budget_usd,
        };
        let end = chat.run().await;
        let Chat {
            records,
            raw,
            cost,
            turns,
            ..
        } = chat;
        server.close().await;
        Ok(finished(records, cost, turns, started, raw, end))
    }
}

/// How the chat loop ended.
struct End {
    subtype: String,
    error: Option<String>,
    timed_out: bool,
}

impl End {
    fn new(subtype: &str) -> Self {
        Self {
            subtype: subtype.to_owned(),
            error: None,
            timed_out: false,
        }
    }

    fn failed(subtype: &str, text: String) -> Self {
        Self {
            subtype: subtype.to_owned(),
            error: Some(text),
            timed_out: false,
        }
    }

    fn timed_out() -> Self {
        Self {
            subtype: "timed_out".to_owned(),
            error: None,
            timed_out: true,
        }
    }
}

#[allow(clippy::cast_possible_truncation)] // milliseconds of one session
fn finished(
    mut records: Vec<Record>,
    cost: Option<f64>,
    turns: u32,
    started: Instant,
    raw: String,
    end: End,
) -> Finished {
    records.push(Record::Result {
        subtype: end.subtype,
        num_turns: Some(turns),
        duration_ms: Some(started.elapsed().as_millis() as u64),
        total_cost_usd: cost,
    });
    Finished {
        records,
        model: None,
        connected: true,
        timed_out: end.timed_out,
        exit_code: None,
        error: end.error,
        raw,
    }
}

/// One session's loop: post the history, run the calls, repeat until the model stops.
struct Chat<'a> {
    endpoint: &'a Endpoint,
    server: &'a Server,
    client: reqwest::Client,
    messages: Vec<Value>,
    tools: Vec<Value>,
    records: Vec<Record>,
    raw: String,
    cost: Option<f64>,
    turns: u32,
    deadline: Instant,
    cap: f64,
}

impl Chat<'_> {
    async fn run(&mut self) -> End {
        loop {
            let reply = match self.next_reply().await {
                Ok(reply) => reply,
                Err(end) => return end,
            };
            if let Some(text) = &reply.text {
                self.records.push(Record::Text { text: text.clone() });
            }
            self.messages.push(reply.message.clone());
            if let Some(cost) = reply.cost_usd {
                self.cost = Some(self.cost.unwrap_or(0.0) + cost);
            }
            if self.cost.is_some_and(|c| c >= self.cap) {
                return End::new("error_max_budget_usd");
            }
            if reply.tool_calls.is_empty() {
                return End::new(match reply.finish_reason.as_deref() {
                    Some("length") => "error_length",
                    Some("content_filter") => "error_content_filter",
                    _ => "success",
                });
            }
            if let Err(end) = self.answer(&reply.tool_calls).await {
                return end;
            }
        }
    }

    /// Posts the history and reads the reply, trying again after rate limits and server errors.
    async fn next_reply(&mut self) -> Result<Reply, End> {
        let body = request(&self.endpoint.model, &self.messages, &self.tools);
        let mut attempt = 0;
        loop {
            let remaining = self.deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(End::timed_out());
            }
            let Ok(response) = tokio::time::timeout(remaining, self.post(&body)).await else {
                return Err(End::timed_out());
            };
            let (status, text) = match response {
                Ok(pair) => pair,
                Err(e) => (0, e.to_string()),
            };
            if (200..300).contains(&status) {
                return self.read(&text);
            }
            if let Some(wait) = retry_after(if status == 0 { 503 } else { status }, attempt) {
                tokio::time::sleep(wait).await;
                attempt += 1;
                continue;
            }
            let what = if status == 0 {
                "request failed".to_owned()
            } else {
                format!("HTTP {status}")
            };
            return Err(End::failed("error_api", format!("{what}: {text}")));
        }
    }

    async fn post(&self, body: &Value) -> reqwest::Result<(u16, String)> {
        let url = format!("{}/chat/completions", self.endpoint.base_url);
        let mut req = self.client.post(url).json(body);
        if let Some(key) = &self.endpoint.api_key {
            req = req.bearer_auth(key);
        }
        let response = req.send().await?;
        let status = response.status().as_u16();
        Ok((status, response.text().await?))
    }

    /// Keeps one reply body in `raw` and reads it.
    fn read(&mut self, text: &str) -> Result<Reply, End> {
        let body: Value = serde_json::from_str(text)
            .map_err(|e| End::failed("error_api", format!("unreadable reply: {e}")))?;
        self.raw.push_str(&body.to_string());
        self.raw.push('\n');
        self.turns += 1;
        parse_reply(&body).map_err(|message| End::failed("error_api", message))
    }

    /// Runs every call of a reply in order, answers each, and sends the pictures after them.
    async fn answer(&mut self, calls: &[ToolCall]) -> Result<(), End> {
        let mut pngs = Vec::new();
        for call in calls {
            self.records.push(Record::ToolUse {
                name: call.name.clone(),
                input: match &call.arguments {
                    Ok(map) => Value::Object(map.clone()),
                    Err(raw) => Value::String(raw.clone()),
                },
            });
            if !is_ours(&call.name) {
                return Err(End::new("error_unknown_tool"));
            }
            let (text, content, is_error) = match &call.arguments {
                Err(_) => {
                    let text = "arguments are not a JSON object".to_owned();
                    (text.clone(), vec![Content::Text { text }], true)
                }
                Ok(map) => {
                    let answer = self
                        .server
                        .call(&call.name, map.clone())
                        .await
                        .map_err(|e| End::failed("error_mcp", format!("{e:#}")))?;
                    pngs.extend(answer.pngs.into_iter().map(|d| (call.name.clone(), d)));
                    (answer.text, answer.content, answer.is_error)
                }
            };
            self.messages.push(tool_message(&call.id, &text));
            self.records.push(Record::ToolResult { content, is_error });
        }
        if !pngs.is_empty() {
            self.messages.push(image_message(&pngs));
        }
        Ok(())
    }
}
