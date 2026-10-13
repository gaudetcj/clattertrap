//! Clattertrap core library. Today it holds `rules`; the simulator modules arrive with milestone 2.
//!
//! Pure library with no I/O. Builds for wasm32.
#![forbid(unsafe_code)]

pub mod events;
mod hash;
pub mod level;
pub mod parts;
pub mod rules;
pub mod scene;
pub mod score;
pub mod sim;
pub mod validate;
mod world;

pub use world::BodyPose;
