//! Running a session's process: what survives when the wall clock kills it.

use super::super::claude::run_session;
use std::time::Duration;

const LINE: &str = r#"{"type":"system","subtype":"init","model":"fake"}"#;

// The child of the test below: this test binary rerun on this one test, which prints a line and
// then outlives any wall clock the test sets.
#[test]
#[ignore = "only run as the child process of a_timed_out_session_keeps_the_output_it_had_printed"]
fn fake_claude_that_prints_then_hangs() {
    println!("{LINE}");
    std::thread::sleep(Duration::from_secs(20));
}

#[tokio::test]
async fn a_timed_out_session_keeps_the_output_it_had_printed() {
    let exe = std::env::current_exe().unwrap();
    let args: Vec<String> = [
        "arena::tests::claude::fake_claude_that_prints_then_hangs",
        "--exact",
        "--ignored",
        "--nocapture",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    let dir = tempfile::tempdir().unwrap();
    let started = std::time::Instant::now();
    let output = run_session(
        exe.to_str().unwrap(),
        &args,
        dir.path(),
        "",
        Duration::from_secs(2),
    )
    .await
    .unwrap();
    assert!(
        output.timed_out,
        "{:?} {:?} {:?}",
        output.exit_code, output.stderr, output.transcript
    );
    assert_eq!(output.exit_code, None);
    assert!(output.transcript.contains(LINE), "{:?}", output.transcript);
    assert!(
        started.elapsed() < Duration::from_secs(15),
        "the kill did not end the wait"
    );
}
