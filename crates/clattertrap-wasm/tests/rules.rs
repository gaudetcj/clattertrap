//! The wrapper exposes the rules version and time step so the viewer can refuse replays made under other rules.

use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn rules_version_and_step_match_core() {
    assert_eq!(
        clattertrap_wasm::rules_version(),
        clattertrap_core::rules::RULES_VERSION
    );
    assert!(
        (clattertrap_wasm::dt_s() - clattertrap_core::rules::Rules::V1.dt_s).abs() < f32::EPSILON
    );
}
