//! The chat-completions JSON the endpoint provider sends and reads: function definitions from the
//! MCP tools, the request body, the parts of a reply, the messages that answer a call, and when a
//! failed request is tried again. No network here.

use serde_json::{Map, Value, json};
use std::time::Duration;

/// One MCP tool as the chat API needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDef {
    /// The tool name.
    pub name: String,
    /// The tool description.
    pub description: String,
    /// The input schema, a JSON Schema object.
    pub parameters: Value,
}

/// One tool call the model made.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    /// The call id the tool message must answer.
    pub id: String,
    /// The tool name.
    pub name: String,
    /// The arguments as an object, or the raw text when they are not one.
    pub arguments: Result<Map<String, Value>, String>,
}

/// What one reply says.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    /// The assistant message verbatim, to echo into the history.
    pub message: Value,
    /// The text the model wrote, when any.
    pub text: Option<String>,
    /// The tool calls, in order.
    pub tool_calls: Vec<ToolCall>,
    /// `stop`, `tool_calls`, `length`, and so on.
    pub finish_reason: Option<String>,
    /// The cost the endpoint reported, when it did.
    pub cost_usd: Option<f64>,
    /// The message of an error the endpoint attached to the choice, when it cut the reply short.
    pub error: Option<String>,
}

/// Function definitions from the tools. `$schema` is dropped: some endpoints reject it.
#[must_use]
pub fn function_tools(tools: &[ToolDef]) -> Vec<Value> {
    tools
        .iter()
        .map(|t| {
            let mut parameters = t.parameters.clone();
            if let Some(obj) = parameters.as_object_mut() {
                obj.remove("$schema");
            }
            json!({
                "type": "function",
                "function": { "name": t.name, "description": t.description, "parameters": parameters }
            })
        })
        .collect()
}

/// The request body; `usage.include` asks an OpenRouter-style endpoint for the cost, and
/// `provider.require_parameters` keeps it off providers that would drop the tools.
#[must_use]
pub fn request(model: &str, messages: &[Value], tools: &[Value]) -> Value {
    json!({
        "model": model,
        "messages": messages,
        "tools": tools,
        "tool_choice": "auto",
        "usage": { "include": true },
        "provider": { "require_parameters": true }
    })
}

/// The parts of a reply body.
///
/// # Errors
/// With the endpoint's message when the body is an error, or when it has no choices.
pub fn parse_reply(body: &Value) -> Result<Reply, String> {
    if let Some(message) = body["error"]["message"].as_str() {
        return Err(message.to_owned());
    }
    let Some(choice) = body["choices"].as_array().and_then(|c| c.first()) else {
        return Err("no choices in the reply".to_owned());
    };
    let message = choice["message"].clone();
    let tool_calls = message["tool_calls"]
        .as_array()
        .map_or_else(Vec::new, |calls| calls.iter().map(tool_call).collect());
    Ok(Reply {
        text: text_of(&message["content"]),
        message,
        tool_calls,
        finish_reason: choice["finish_reason"].as_str().map(ToOwned::to_owned),
        cost_usd: body["usage"]["cost"].as_f64(),
        error: choice["error"]["message"].as_str().map(ToOwned::to_owned),
    })
}

/// The text of a message's content, a string or text parts.
fn text_of(content: &Value) -> Option<String> {
    let text = match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter(|p| p["type"] == "text")
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };
    (!text.is_empty()).then_some(text)
}

fn tool_call(call: &Value) -> ToolCall {
    let raw = call["function"]["arguments"].as_str().unwrap_or("");
    let arguments = match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err(raw.to_owned()),
    };
    ToolCall {
        id: call["id"].as_str().unwrap_or("").to_owned(),
        name: call["function"]["name"].as_str().unwrap_or("").to_owned(),
        arguments,
    }
}

/// The tool message answering one call.
#[must_use]
pub fn tool_message(id: &str, text: &str) -> Value {
    json!({ "role": "tool", "tool_call_id": id, "content": text })
}

/// One user message carrying every picture of a batch, each named by its tool, as data URLs.
#[must_use]
pub fn image_message(pngs: &[(String, String)]) -> Value {
    let mut parts = Vec::new();
    for (tool, data) in pngs {
        parts.push(json!({ "type": "text", "text": format!("The picture from {tool}:") }));
        parts.push(json!({
            "type": "image_url",
            "image_url": { "url": format!("data:image/png;base64,{data}") }
        }));
    }
    json!({ "role": "user", "content": parts })
}

/// How long to wait before trying again after `status` on `attempt` (from 0): rate limits and
/// server errors get 1, 2, and 4 s; anything else, or a fourth failure, is final.
#[must_use]
pub fn retry_after(status: u16, attempt: u32) -> Option<Duration> {
    let again = status == 429 || (500..=599).contains(&status);
    (again && attempt < 3).then(|| Duration::from_secs(1 << attempt))
}
