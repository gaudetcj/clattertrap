//! The files under `pilot/`: the contestant prompt every model gets and the hand-use MCP config.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::repo_root;

#[test]
fn the_prompt_has_exactly_the_level_and_run_budget_placeholders() {
    let text = std::fs::read_to_string(repo_root().join("pilot").join("prompt.txt")).unwrap();
    let levels = text.matches("{level}").count();
    let budgets = text.matches("{run_budget}").count();
    assert!(levels >= 1, "{text}");
    assert_eq!(budgets, 1, "{text}");
    assert_eq!(text.matches('{').count(), levels + budgets, "{text}");
    assert_eq!(text.matches('}').count(), levels + budgets, "{text}");
    assert!(text.contains("submit"), "{text}");
    assert!(text.ends_with('\n'), "{text}");
}

#[test]
fn the_mcp_config_starts_the_mcp_subcommand_in_the_workshop_arm() {
    let text = std::fs::read_to_string(repo_root().join("pilot").join("mcp.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    let server = &v["mcpServers"]["clattertrap"];
    assert!(server["command"].as_str().is_some_and(|c| !c.is_empty()));
    let args: Vec<&str> = server["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap())
        .collect();
    let at = args.iter().position(|a| *a == "mcp").unwrap();
    assert_eq!(args[at + 1..], ["--arm", "workshop", "--out", "pilot/scratch"]);
}
