//! Clattertrap core library. Today it holds `rules`; the simulator modules arrive with milestone 2.
//!
//! Pure library with no I/O. Builds for wasm32.
#![forbid(unsafe_code)]

pub mod level;
pub mod rules;
pub mod scene;
