//! Clattertrap core library: level and scene formats with their JSON Schemas, validation, every v0.1
//! part, the rapier world, the simulator, events, scoring, and a PNG renderer for the `look` tool.
//!
//! Pure library with no I/O. Builds for wasm32.
#![forbid(unsafe_code)]

pub mod events;
mod goal;
mod hash;
pub mod level;
pub mod parts;
pub mod render;
pub mod rules;
pub mod scene;
pub mod score;
pub mod sim;
pub mod validate;
mod world;

pub use world::{BodyPose, Slot};
