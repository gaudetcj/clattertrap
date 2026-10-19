//! How a run's status follows from what the provider left behind, whichever provider it was.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::super::provider::Finished;
use super::super::status::{foreign_tools, is_ours, status};
use crate::records::Status;
use crate::records::transcript::Record;
use serde_json::json;

fn result(subtype: &str) -> Record {
    Record::Result {
        subtype: subtype.to_owned(),
        num_turns: Some(1),
        duration_ms: Some(10),
        total_cost_usd: Some(0.1),
    }
}

fn tool_use(name: &str) -> Record {
    Record::ToolUse {
        name: name.to_owned(),
        input: json!({}),
    }
}

fn finished(records: Vec<Record>) -> Finished {
    Finished {
        records,
        model: None,
        connected: true,
        timed_out: false,
        exit_code: Some(0),
        error: None,
        raw: String::new(),
    }
}

#[test]
fn our_tools_are_known_bare_or_with_the_claude_code_prefix() {
    for ok in [
        "rules",
        "submit",
        "mcp__clattertrap__check",
        "mcp__clattertrap__level",
    ] {
        assert!(is_ours(ok), "{ok}");
    }
    for bad in [
        "Bash",
        "mcp__other__rules",
        "mcp__clattertrap__",
        "mcp__clattertrap__bash",
        "",
    ] {
        assert!(!is_ours(bad), "{bad}");
    }
}

#[test]
fn a_session_that_ends_without_a_submission_is_no_submission() {
    let f = finished(vec![tool_use("mcp__clattertrap__rules"), result("success")]);
    assert_eq!(status(&f, false), (Status::NoSubmission, None));
    assert_eq!(status(&f, true), (Status::Submitted, None));
}

#[test]
fn a_submission_before_the_wall_clock_counts_and_after_it_times_out() {
    let mut f = finished(vec![]);
    f.timed_out = true;
    assert_eq!(status(&f, true), (Status::Submitted, None));
    let (s, why) = status(&f, false);
    assert_eq!(s, Status::TimedOut);
    assert_eq!(why.as_deref(), Some("the wall clock ran out"));
}

#[test]
fn the_budget_cap_is_over_budget_and_other_error_subtypes_abort() {
    let (s, _) = status(&finished(vec![result("error_max_budget_usd")]), false);
    assert_eq!(s, Status::OverBudget);
    let mut f = finished(vec![result("error_during_execution")]);
    f.error = Some("API Error: 529 overloaded".to_owned());
    assert_eq!(
        status(&f, false),
        (
            Status::Aborted,
            Some("error_during_execution: API Error: 529 overloaded".to_owned())
        )
    );
    let mut f = finished(vec![result("success")]);
    f.error = Some("the socket closed".to_owned());
    assert_eq!(status(&f, false).0, Status::Aborted);
}

#[test]
fn a_missing_result_or_a_server_that_never_connected_aborts() {
    let (s, why) = status(&finished(vec![]), false);
    assert_eq!(s, Status::Aborted);
    assert_eq!(why.as_deref(), Some("no result record"));
    let mut f = finished(vec![result("success")]);
    f.connected = false;
    assert_eq!(
        status(&f, false),
        (
            Status::Aborted,
            Some("the clattertrap MCP server did not connect".to_owned())
        )
    );
}

#[test]
fn a_tool_outside_the_server_flags_the_run_even_with_a_submission() {
    let f = finished(vec![
        tool_use("mcp__clattertrap__check"),
        tool_use("Bash"),
        tool_use("Bash"),
        tool_use("Read"),
        result("success"),
    ]);
    assert_eq!(foreign_tools(&f.records), ["Bash", "Read"]);
    assert_eq!(
        status(&f, true),
        (Status::Flagged, Some("called Bash".to_owned()))
    );
}
