//! The wrapper exposes the rules version so the viewer can refuse replays made under other rules.

#[test]
fn rules_version_matches_core() {
    assert_eq!(
        clattertrap_wasm::rules_version(),
        clattertrap_core::rules::RULES_VERSION
    );
}
