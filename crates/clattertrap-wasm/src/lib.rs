//! wasm-bindgen wrapper over `clattertrap-core` for the browser viewer. No logic lives here.
#![forbid(unsafe_code)]

use wasm_bindgen::prelude::wasm_bindgen;

/// The rules version this build simulates under.
#[wasm_bindgen]
#[must_use]
pub fn rules_version() -> u32 {
    clattertrap_core::rules::RULES_VERSION
}
