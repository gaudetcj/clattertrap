//! The rules as contestants read them, in centimetres and degrees, built from `Rules` so the text and
//! the simulator cannot drift apart.

use clattertrap_core::rules::{MaterialProps, Range, Rules};
use std::f32::consts::PI;

/// Metres to centimetres, rounded to a tenth.
fn cm(m: f32) -> f32 {
    libm::roundf(m * 1000.0) / 10.0
}

fn range(r: Range) -> String {
    format!("{}..{}", cm(r.min_m), cm(r.max_m))
}

/// Mass of a ball of this material in kilograms.
fn ball_mass_kg(rules: &Rules, m: MaterialProps) -> f32 {
    m.density * PI * rules.ball_radius_m * rules.ball_radius_m
}

/// Speed a resting body of `mass_kg` leaves a spring pad with, and how high it would rise.
fn launch(rules: &Rules, mass_kg: f32) -> (f32, f32) {
    let v = rules.spring_impulse_ns / mass_kg;
    (v, cm(v * v / (2.0 * rules.gravity_m_s2)))
}

/// The complete rules text.
pub fn rules_text(rules: &Rules) -> String {
    let steel = ball_mass_kg(rules, rules.steel);
    let wood = ball_mass_kg(rules, rules.wood);
    let rubber = ball_mass_kg(rules, rules.rubber);
    let (wood_v, wood_h) = launch(rules, wood);
    let (steel_v, steel_h) = launch(rules, steel);
    let rise = rules.gravity_m_s2 * -rules.balloon_gravity_scale / rules.balloon_damping;
    let [dom_w, dom_h] = rules.domino_size_m.map(cm);
    let [pad_w, pad_h] = rules.spring_pad_size_m.map(cm);
    let [pin_w, pin_h] = rules.pin_size_m.map(cm);
    let [cap_w, cap_h] = rules.button_cap_size_m.map(cm);
    let lines = [
        format!("Clattertrap rules v{}", rules.version),
        String::new(),
        "World".to_owned(),
        "- Units are centimetres and degrees. The origin is bottom-left, x runs right, y runs up. A part's position is its centre unless the table says otherwise.".to_owned(),
        "- Angles are counterclockwise positive. angle_deg 0 is horizontal; a beam at -15 slopes down to the right.".to_owned(),
        format!("- Gravity is {} cm/s^2 downward. Everything starts at rest and physics starts at once, so anything in the air falls.", cm(rules.gravity_m_s2)),
        format!("- The floor is solid. There are no walls and no ceiling: a part whose centre goes more than {} cm outside the level rectangle is removed.", cm(rules.leave_margin_m)),
        format!("- Time advances in steps of 1/{} s. A run ends when the goal is met, when the level's time limit passes, or when everything has stopped moving.", libm::roundf(1.0 / rules.dt_s)),
        String::new(),
        "Materials".to_owned(),
        "| material | mass of a ball | bounce | friction |".to_owned(),
        format!("| steel | {steel:.2} kg | {} | {} |", rules.steel.restitution, rules.steel.friction),
        format!("| wood | {wood:.2} kg | {} | {} |", rules.wood.restitution, rules.wood.friction),
        format!("| rubber | {rubber:.2} kg | {} | {} |", rules.rubber.restitution, rules.rubber.friction),
        "Bounce between two bodies is the larger of the pair; friction is the average. Beams, dominoes, crates, seesaws, and buckets are wood.".to_owned(),
        String::new(),
        "Parts you may place".to_owned(),
        "The level's inventory says how many of each kind. Kinds it does not list are not available.".to_owned(),
        "| kind | fields | size | behaviour |".to_owned(),
        format!("| beam | x_cm, y_cm, length_cm ({}), angle_deg | {} cm thick | fixed wood plank |", range(rules.beam_length), cm(rules.plank_thickness_m)),
        format!("| funnel | x_cm, y_cm (throat centre), mouth_cm ({}), throat_cm ({}..mouth), height_cm ({}) | {} cm walls | fixed; two walls slope in from the mouth down to the throat |", range(rules.funnel_mouth), cm(rules.funnel_throat_min_m), range(rules.funnel_height), cm(rules.wall_thickness_m)),
        format!("| ball | x_cm, y_cm, material (steel, wood, rubber) | radius {} cm | rolls and bounces as its material does |", cm(rules.ball_radius_m)),
        format!("| domino | x_cm, y_cm, angle_deg (default 0) | {dom_w} by {dom_h} cm | wood tile standing on end; tips when pushed |"),
        format!("| crate | x_cm, y_cm, side_cm ({}) | square | wood box |", range(rules.crate_side)),
        format!("| seesaw | x_cm, y_cm, length_cm ({}), angle_deg (default 0) | {} cm thick | wood plank that pivots freely about its centre, which is fixed in space |", range(rules.seesaw_length), cm(rules.plank_thickness_m)),
        format!("| pendulum | x_cm, y_cm (the pin), length_cm ({}), angle_deg (default 0, measured from straight down) | steel weight of radius {} cm | swings on a rigid rope from the pin and is released at once |", range(rules.pendulum_length), cm(rules.pendulum_radius_m)),
        format!("| spring_pad | x_cm, y_cm, angle_deg (default 0) | {pad_w} by {pad_h} cm | fixed plate; a body that lands on it gets a kick of {} N s along the plate's normal: a wood ball leaves at {wood_v:.1} m/s and rises about {wood_h:.0} cm, a steel ball at {steel_v:.1} m/s rises {steel_h:.0} cm |", rules.spring_impulse_ns),
        format!("| bucket | x_cm, y_cm (centre of the floor's underside), width_cm ({}), height_cm ({}) | {} cm walls | wood open box; it can move |", range(rules.bucket_size), range(rules.bucket_size), cm(rules.wall_thickness_m)),
        format!("| balloon | x_cm, y_cm | radius {} cm | rises at up to {rise:.1} m/s and slides along the underside of anything above it; a pin pops it |", cm(rules.balloon_radius_m)),
        format!("| pin | x_cm, y_cm (the base), angle_deg (default -90, pointing down) | {pin_w} by {pin_h} cm | fixed needle |"),
        format!("| rope | a, b, max_length_cm ({}) | no body | ties the centres of parts a and b; slack until taut, then keeps them within that length |", range(rules.rope_length)),
        String::new(),
        "Goal objects, which only a level places".to_owned(),
        format!("| bell | brass disc of radius {} cm hung below its anchor | rings when hit hard enough: a wood ball at {:.1} m/s or a steel ball at {:.1} m/s |", cm(rules.bell_radius_m), rules.bell_min_impulse_ns / wood, rules.bell_min_impulse_ns / steel),
        format!("| button | {cap_w} by {cap_h} cm cap on a spring | pressed when pushed down its full travel and held there |"),
        format!("| basket | open box with {} cm walls | counts objects that come to rest inside it |", cm(rules.basket_wall_m)),
        String::new(),
        "Goals".to_owned(),
        "ring, press, fill (count objects at rest inside the basket), or pop. A goal may list via: part ids that must be touched in that order before the goal counts.".to_owned(),
        String::new(),
        "Placement rules".to_owned(),
        "- Every placed part's centre must lie inside a build zone. Fixed objects are exempt.".to_owned(),
        "- Nothing may overlap at the start: not another part, a fixed object, or the floor. Leave a gap of a few millimetres.".to_owned(),
        "- Every part has a unique id; the fixed objects' ids are taken.".to_owned(),
        "- Lengths outside the ranges above are rejected. check lists every problem with its JSON path and a fix.".to_owned(),
        String::new(),
        "Scene JSON".to_owned(),
        r#"{"level": "<level id>", "parts": [{"id": "example", "kind": "beam", "x_cm": 120, "y_cm": 210, "length_cm": 120, "angle_deg": -15}]}"#.to_owned(),
        "Field names carry their units. The full schema: clattertrap rules --schema".to_owned(),
    ];
    let mut text = lines.join("\n");
    text.push('\n');
    text
}
