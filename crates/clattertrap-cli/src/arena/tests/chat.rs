//! The chat-completions JSON the endpoint provider sends and reads, without any network.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::super::chat::{
    ToolDef, function_tools, image_message, parse_reply, request, retry_after, tool_message,
};
use serde_json::{Value, json};
use std::time::Duration;

fn defs() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "check".to_owned(),
            description: "Validate a scene.".to_owned(),
            parameters: json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": { "scene": { "description": "The scene." } },
                "required": ["scene"]
            }),
        },
        ToolDef {
            name: "levels".to_owned(),
            description: "Every level.".to_owned(),
            parameters: json!({ "type": "object", "properties": {} }),
        },
    ]
}

#[test]
fn tools_become_function_definitions_without_the_schema_key() {
    let tools = function_tools(&defs());
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "check");
    assert_eq!(tools[0]["function"]["description"], "Validate a scene.");
    assert_eq!(
        tools[0]["function"]["parameters"]["required"],
        json!(["scene"])
    );
    assert!(tools[0]["function"]["parameters"].get("$schema").is_none());
    assert_eq!(
        tools[1]["function"]["parameters"],
        json!({ "type": "object", "properties": {} })
    );
}

#[test]
fn the_request_carries_the_model_messages_tools_and_asks_for_usage() {
    let messages = vec![
        json!({ "role": "system", "content": "Build a machine." }),
        json!({ "role": "user", "content": "Your level is bell-drop." }),
    ];
    let body = request("openai/gpt-5-mini", &messages, &function_tools(&defs()));
    assert_eq!(body["model"], "openai/gpt-5-mini");
    assert_eq!(body["messages"], Value::Array(messages));
    assert_eq!(body["tool_choice"], "auto");
    assert_eq!(body["usage"]["include"], true);
    insta::assert_snapshot!(serde_json::to_string_pretty(&body).unwrap());
}

#[test]
fn a_reply_with_parallel_tool_calls_keeps_the_message_and_parses_the_arguments() {
    let body = json!({
        "id": "gen-1",
        "model": "openai/gpt-5-mini",
        "choices": [{
            "index": 0,
            "finish_reason": "tool_calls",
            "message": {
                "role": "assistant",
                "content": "Reading the rules and the level.",
                "tool_calls": [
                    { "id": "call_a", "type": "function", "function": { "name": "rules", "arguments": "{}" } },
                    { "id": "call_b", "type": "function", "function": { "name": "level", "arguments": "{\"id\":\"bell-drop\"}" } },
                    { "id": "call_c", "type": "function", "function": { "name": "check", "arguments": "{\"scene\": [" } }
                ]
            }
        }],
        "usage": { "prompt_tokens": 10, "completion_tokens": 5, "cost": 0.0125 }
    });
    let reply = parse_reply(&body).unwrap();
    assert_eq!(reply.message, body["choices"][0]["message"]);
    assert_eq!(
        reply.text.as_deref(),
        Some("Reading the rules and the level.")
    );
    assert_eq!(reply.finish_reason.as_deref(), Some("tool_calls"));
    assert_eq!(reply.cost_usd, Some(0.0125));
    assert_eq!(reply.tool_calls.len(), 3);
    assert_eq!(reply.tool_calls[0].id, "call_a");
    assert_eq!(reply.tool_calls[0].name, "rules");
    assert_eq!(reply.tool_calls[0].arguments, Ok(serde_json::Map::new()));
    assert_eq!(
        reply.tool_calls[1].arguments.as_ref().unwrap()["id"],
        "bell-drop"
    );
    assert_eq!(
        reply.tool_calls[2].arguments,
        Err("{\"scene\": [".to_owned())
    );
}

#[test]
fn a_final_reply_has_no_tool_calls_and_text_may_come_as_parts() {
    let body = json!({
        "choices": [{
            "finish_reason": "stop",
            "message": { "role": "assistant", "content": [{ "type": "text", "text": "Done." }, { "type": "text", "text": "Bye." }] }
        }],
        "usage": { "prompt_tokens": 1, "completion_tokens": 1 }
    });
    let reply = parse_reply(&body).unwrap();
    assert!(reply.tool_calls.is_empty());
    assert_eq!(reply.text.as_deref(), Some("Done.\nBye."));
    assert_eq!(reply.finish_reason.as_deref(), Some("stop"));
    assert_eq!(reply.cost_usd, None);
    let empty =
        parse_reply(&json!({ "choices": [{ "message": { "role": "assistant", "content": "" } }] }))
            .unwrap();
    assert_eq!(empty.text, None);
    assert_eq!(empty.finish_reason, None);
}

#[test]
fn an_error_body_or_a_reply_without_choices_is_an_error() {
    let err = parse_reply(&json!({ "error": { "message": "model not found", "code": 404 } }));
    assert_eq!(err.unwrap_err(), "model not found");
    let none = parse_reply(&json!({ "choices": [] }));
    assert_eq!(none.unwrap_err(), "no choices in the reply");
    let junk = parse_reply(&json!("nonsense"));
    assert_eq!(junk.unwrap_err(), "no choices in the reply");
}

#[test]
fn tool_and_image_messages_have_the_chat_shapes() {
    assert_eq!(
        tool_message("call_a", "ok: 2 parts placed"),
        json!({ "role": "tool", "tool_call_id": "call_a", "content": "ok: 2 parts placed" })
    );
    let msg = image_message(&[("look".to_owned(), "iVBORw0KGgo=".to_owned())]);
    assert_eq!(msg["role"], "user");
    assert_eq!(
        msg["content"][0],
        json!({ "type": "text", "text": "The picture from look:" })
    );
    assert_eq!(
        msg["content"][1],
        json!({ "type": "image_url", "image_url": { "url": "data:image/png;base64,iVBORw0KGgo=" } })
    );
    assert_eq!(msg["content"].as_array().unwrap().len(), 2);
}

#[test]
fn only_rate_limits_and_server_errors_are_retried_three_times() {
    assert_eq!(retry_after(429, 0), Some(Duration::from_secs(1)));
    assert_eq!(retry_after(503, 1), Some(Duration::from_secs(2)));
    assert_eq!(retry_after(500, 2), Some(Duration::from_secs(4)));
    assert_eq!(retry_after(500, 3), None);
    assert_eq!(retry_after(400, 0), None);
    assert_eq!(retry_after(401, 0), None);
    assert_eq!(retry_after(200, 0), None);
}
