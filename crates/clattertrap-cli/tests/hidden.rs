//! Invariant 7: reference solutions never reach contestants through `rules` or `level`.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::scene::Scene;
use common::{level_dirs, levels_dir, stdout};

#[test]
fn no_tool_output_contains_a_reference_placement() {
    let rules = stdout(&["rules"]);
    for id in level_dirs() {
        let reference =
            std::fs::read_to_string(levels_dir().join(&id).join("reference.json")).unwrap();
        let scene: Scene = serde_json::from_str(&reference).unwrap();
        assert!(!scene.parts.is_empty(), "{id} has an empty reference");
        let level = stdout(&["level", &id]);
        for part in &scene.parts {
            let compact = serde_json::to_string(part).unwrap();
            let ids = [
                format!("\"id\": \"{}\"", part.id),
                format!("\"id\":\"{}\"", part.id),
            ];
            for (name, text) in [("level", &level), ("rules", &rules)] {
                assert!(
                    !text.contains(&compact),
                    "{id}: {name} prints a reference part"
                );
                assert!(
                    ids.iter().all(|q| !text.contains(q)),
                    "{id}: {name} prints reference id {:?}",
                    part.id
                );
            }
        }
    }
}
