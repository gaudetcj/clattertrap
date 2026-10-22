//! Numeric range and finiteness checks for every part parameter, in centimetres.

use super::Problem;
use super::checks::all;
use crate::level::Level;
use crate::rules::{Range, Rules};
use crate::scene::{PartSpec, Scene};

fn cm(v_m: f32) -> f32 {
    libm::roundf(v_m * 1000.0) / 10.0
}

fn check_range(path: &str, field: &str, value_cm: f32, range: Range, out: &mut Vec<Problem>) {
    let (lo, hi) = (cm(range.min_m), cm(range.max_m));
    if !value_cm.is_finite() || value_cm < lo || value_cm > hi {
        out.push(Problem::new(
            path,
            format!("{field} {value_cm} is outside {lo}..{hi}; set {field} within that range"),
        ));
    }
}

fn check_finite(path: &str, field: &str, value: f32, out: &mut Vec<Problem>) {
    if !value.is_finite() {
        out.push(Problem::new(
            path,
            format!("{field} is {value}; use a finite number"),
        ));
    }
}

pub(super) fn ranges(level: &Level, scene: &Scene, rules: &Rules, out: &mut Vec<Problem>) {
    for (path, p) in all(level, scene) {
        if let Some([x, y]) = p.spec.centre_cm() {
            check_finite(&path, "x_cm", x, out);
            check_finite(&path, "y_cm", y, out);
        }
        match &p.spec {
            PartSpec::Beam {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.beam_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Funnel {
                mouth_cm,
                throat_cm,
                height_cm,
                ..
            } => {
                check_range(&path, "mouth_cm", *mouth_cm, rules.funnel_mouth, out);
                let throat = Range {
                    min_m: rules.funnel_throat_min_m,
                    max_m: *mouth_cm / 100.0,
                };
                check_range(&path, "throat_cm", *throat_cm, throat, out);
                check_range(&path, "height_cm", *height_cm, rules.funnel_height, out);
            }
            PartSpec::Crate { side_cm, .. } => {
                check_range(&path, "side_cm", *side_cm, rules.crate_side, out);
            }
            PartSpec::Seesaw {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.seesaw_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Pendulum {
                length_cm,
                angle_deg,
                ..
            } => {
                check_range(&path, "length_cm", *length_cm, rules.pendulum_length, out);
                check_finite(&path, "angle_deg", *angle_deg, out);
            }
            PartSpec::Bucket {
                width_cm,
                height_cm,
                ..
            } => {
                check_range(&path, "width_cm", *width_cm, rules.bucket_size, out);
                check_range(&path, "height_cm", *height_cm, rules.bucket_size, out);
            }
            PartSpec::Rope { max_length_cm, .. } => check_range(
                &path,
                "max_length_cm",
                *max_length_cm,
                rules.rope_length,
                out,
            ),
            PartSpec::Bell { rope_cm, .. } => {
                check_range(&path, "rope_cm", *rope_cm, rules.bell_rope, out);
            }
            PartSpec::Button { travel_cm, .. } => {
                check_range(&path, "travel_cm", *travel_cm, rules.button_travel, out);
            }
            PartSpec::Basket {
                inner_width_cm,
                inner_height_cm,
                ..
            } => {
                check_range(
                    &path,
                    "inner_width_cm",
                    *inner_width_cm,
                    rules.bucket_size,
                    out,
                );
                check_range(
                    &path,
                    "inner_height_cm",
                    *inner_height_cm,
                    rules.bucket_size,
                    out,
                );
            }
            _ => angles(&path, &p.spec, out),
        }
    }
}

/// The finiteness checks of the kinds with only an angle, and a portal's second end.
fn angles(path: &str, spec: &PartSpec, out: &mut Vec<Problem>) {
    match spec {
        PartSpec::Portal {
            angle_deg,
            blue_x_cm,
            blue_y_cm,
            blue_angle_deg,
            ..
        } => {
            check_finite(path, "angle_deg", *angle_deg, out);
            check_finite(path, "blue_x_cm", *blue_x_cm, out);
            check_finite(path, "blue_y_cm", *blue_y_cm, out);
            check_finite(path, "blue_angle_deg", *blue_angle_deg, out);
        }
        PartSpec::Domino { angle_deg, .. }
        | PartSpec::SpringPad { angle_deg, .. }
        | PartSpec::Pin { angle_deg, .. } => {
            check_finite(path, "angle_deg", *angle_deg, out);
        }
        _ => {}
    }
}
