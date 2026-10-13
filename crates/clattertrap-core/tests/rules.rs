//! Rules version pin, see CLAUDE.md invariant 5.

#[test]
fn rules_version_is_1_until_the_first_gallery_entry() {
    assert_eq!(clattertrap_core::rules::RULES_VERSION, 1);
}
