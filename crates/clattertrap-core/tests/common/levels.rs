//! Every level in `levels/` with its reference solution and committed golden lines, embedded so the
//! tests also run under wasmtime, which cannot read the directory.

/// Level id, level JSON, reference JSON, and the golden lines for the reference and the empty scene.
pub struct Fixture {
    pub id: &'static str,
    pub level: &'static str,
    pub reference: &'static str,
    pub reference_snap: &'static str,
    pub empty_snap: &'static str,
}

macro_rules! fixture {
    ($id:literal) => {
        Fixture {
            id: $id,
            level: include_str!(concat!("../../../../levels/", $id, "/level.json")),
            reference: include_str!(concat!("../../../../levels/", $id, "/reference.json")),
            reference_snap: include_str!(concat!(
                "../snapshots/fixtures__",
                $id,
                "-reference.snap"
            )),
            empty_snap: include_str!(concat!("../snapshots/fixtures__", $id, "-empty.snap")),
        }
    };
}

/// Every level. A directory added under `levels/` is added here too; a test checks the two agree.
pub const ALL: [Fixture; 12] = [
    fixture!("bell-drop"),
    fixture!("balloon-pin"),
    fixture!("basket-fill"),
    fixture!("button-drop"),
    fixture!("domino-run"),
    fixture!("funnel-basket"),
    fixture!("grand-tour"),
    fixture!("pendulum-bell"),
    fixture!("post-first"),
    fixture!("rope-swing"),
    fixture!("seesaw-launch"),
    fixture!("spring-launch"),
];
