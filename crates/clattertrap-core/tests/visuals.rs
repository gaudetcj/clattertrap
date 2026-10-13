//! Every body reports a pose and a drawing description so the viewer can draw it, and body slots keep
//! their order after a body is removed.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use clattertrap_core::level::Goal;
use clattertrap_core::parts::{Look, Visual};
use clattertrap_core::rules::Rules;
use clattertrap_core::scene::Material;
use common::{ball, balloon, beam, level, level_with_goal, part, pin, scene, seesaw, sim};

#[test]
fn slots_pair_poses_with_visuals_in_insertion_order() {
    let lv = level(vec![ball("k", 100.0, 200.0, Material::Rubber)]);
    let s = sim(
        &lv,
        &scene(vec![
            beam("b", 200.0, 100.0, 80.0, 0.0),
            seesaw("s", 300.0, 50.0, 100.0, 0.0),
        ]),
    );
    let slots: Vec<_> = s.slots().collect();
    let r = Rules::V1;
    assert_eq!(
        slots.len(),
        7,
        "floor, ball, bell, bell anchor, beam, plank, pivot"
    );
    assert_eq!(slots[1].part, part(&s, "k"));
    assert_eq!(slots[1].pose.unwrap().part, part(&s, "k"));
    assert_eq!(
        slots[1].visual,
        Visual::Circle {
            r_m: r.ball_radius_m,
            look: Look::Rubber
        }
    );
    assert_eq!(
        slots[2].visual,
        Visual::Circle {
            r_m: r.bell_radius_m,
            look: Look::Brass
        }
    );
    assert_eq!(slots[3].visual, Visual::Pivot);
    assert_eq!(
        slots[4].visual,
        Visual::Box {
            w_m: 0.8,
            h_m: r.plank_thickness_m,
            look: Look::Wood
        }
    );
    assert_eq!(slots[5].part, part(&s, "s"));
    assert!(matches!(
        slots[5].visual,
        Visual::Box {
            look: Look::Wood,
            ..
        }
    ));
    assert_eq!(slots[6].visual, Visual::Pivot);
    let pivot = slots[6].pose.unwrap();
    assert!((pivot.x_m - 3.0).abs() < 1e-6 && (pivot.y_m - 0.5).abs() < 1e-6);
}

#[test]
fn a_popped_balloon_keeps_its_slot_with_no_pose() {
    let goal = Goal::Pop {
        target: "bl".to_owned(),
        via: vec![],
    };
    let lv = level_with_goal(
        vec![balloon("bl", 200.0, 50.0), pin("needle", 200.0, 290.0)],
        goal,
    );
    let mut s = sim(&lv, &scene(vec![]));
    let before: Vec<_> = s.slots().map(|slot| (slot.part, slot.visual)).collect();
    while s.finished().is_none() {
        s.step();
    }
    let after: Vec<_> = s.slots().collect();
    assert_eq!(
        before,
        after
            .iter()
            .map(|slot| (slot.part, slot.visual))
            .collect::<Vec<_>>(),
        "slot order and visuals survive a removal"
    );
    let gone: Vec<_> = after.iter().filter(|slot| slot.pose.is_none()).collect();
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0].part, part(&s, "bl"));
    assert!(matches!(
        gone[0].visual,
        Visual::Circle {
            look: Look::Balloon,
            ..
        }
    ));
}
