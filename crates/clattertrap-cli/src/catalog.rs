//! The levels built into the binary, so every command works from any directory. Reference solutions
//! are not embedded; nothing in the binary needs them.

use anyhow::{Result, anyhow};
use clattertrap_core::level::Level;
use clattertrap_core::validate::parse_level;

/// One level's id and JSON.
pub struct Entry {
    /// Directory name under `levels/`.
    pub id: &'static str,
    /// The level file.
    pub level_json: &'static str,
}

macro_rules! entry {
    ($id:literal) => {
        Entry {
            id: $id,
            level_json: include_str!(concat!("../../../levels/", $id, "/level.json")),
        }
    };
}

/// Every level, in directory order.
pub const ALL: [Entry; 4] = [
    entry!("balloon-pin"),
    entry!("basket-fill"),
    entry!("bell-drop"),
    entry!("button-drop"),
];

/// The entry with this id.
///
/// # Errors
/// When no level has the id; the message says how to list them.
pub fn find(id: &str) -> Result<&'static Entry> {
    ALL.iter()
        .find(|e| e.id == id)
        .ok_or_else(|| anyhow!("no level {id:?}; run `clattertrap levels` to see the ids"))
}

/// Parses an entry's level.
///
/// # Errors
/// When the embedded JSON does not parse, which a test catches before release.
pub fn level(entry: &Entry) -> Result<Level> {
    parse_level(entry.level_json).map_err(|p| anyhow!("levels/{}/level.json: {p}", entry.id))
}

/// Every level, sorted by tier and then id.
///
/// # Errors
/// When any embedded level does not parse.
pub fn levels() -> Result<Vec<Level>> {
    let mut levels = ALL.iter().map(level).collect::<Result<Vec<_>>>()?;
    levels.sort_by(|a, b| a.tier.cmp(&b.tier).then_with(|| a.id.cmp(&b.id)));
    Ok(levels)
}
