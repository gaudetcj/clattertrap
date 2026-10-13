//! Every body reports a pose and a drawing description so the viewer can draw it.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::parts::{Look, Visual};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use common::{ball, beam, level, part, scene, seesaw, sim};

#[test]
fn bodies_pair_poses_with_visuals_in_insertion_order() {
    let lv = level(vec![ball("k", 100.0, 200.0, Material::Rubber)]);
    let s = sim(
        &lv,
        &scene(vec![
            beam("b", 200.0, 100.0, 80.0, 0.0),
            seesaw("s", 300.0, 50.0, 100.0, 0.0),
        ]),
    );
    let bodies: Vec<_> = s.bodies().collect();
    let r = Rules::V1;
    assert_eq!(bodies.len(), 5, "floor, ball, beam, plank, pivot");
    assert_eq!(bodies[1].0.part, part(&s, "k"));
    assert_eq!(
        bodies[1].1,
        Visual::Circle {
            r_m: r.ball_radius_m,
            look: Look::Rubber
        }
    );
    assert_eq!(
        bodies[2].1,
        Visual::Box {
            w_m: 0.8,
            h_m: r.plank_thickness_m,
            look: Look::Wood
        }
    );
    assert_eq!(bodies[3].0.part, part(&s, "s"));
    assert!(matches!(
        bodies[3].1,
        Visual::Box {
            look: Look::Wood,
            ..
        }
    ));
    assert_eq!(bodies[4].1, Visual::Pivot);
    assert!((bodies[4].0.x_m - 3.0).abs() < 1e-6 && (bodies[4].0.y_m - 0.5).abs() < 1e-6);
}
