use std::cmp::{max, min};
use units::Unit;

// Ensure that a value is between lower and an upper
pub fn bound(value: usize, lower: usize, upper: usize) -> usize {
    min(upper, max(lower, value))
}

// A change to hit function based on a fairly simple sigmoid curve.
pub fn chance_to_hit(from: &Unit, target: &Unit) -> f32 {
    let distance = (from.x as f32 - target.x as f32).hypot(from.y as f32 - target.y as f32);

    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}