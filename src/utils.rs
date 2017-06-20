use std::cmp::{max, min};

// Ensure that a value is between lower and an upper
pub fn bound(value: usize, lower: usize, upper: usize) -> usize {
    min(upper, max(lower, value))
}

// Ensure that a floating point value is between lower and upper as usize
pub fn bound_float(value: f32, lower: usize, upper: usize) -> usize {
    let value = value.round();
    let value = if value < 0.0 { 0 } else { value as usize };

    bound(value, lower, upper)
}

// Calculate the distance between two items on the map
pub fn distance(from_x: usize, from_y: usize, target_x: usize, target_y: usize) -> f32 {
    (from_x as f32 - target_x as f32).hypot(from_y as f32 - target_y as f32)
}

// Caculate if a distance between two points is below a given value
pub fn distance_under(from_x: usize, from_y: usize, target_x: usize, target_y: usize, value: f32) -> bool {
    // First test the linear distances
    (from_x as f32 - target_x as f32).abs() <= value &&
    (from_y as f32 - target_y as f32).abs() <= value &&
    // Then the hypot distance
    distance(from_x, from_y, target_x, target_y) <= value
}

// A change to hit function based on a fairly simple sigmoid curve.
pub fn chance_to_hit(from_x: usize, from_y: usize, target_x: usize, target_y: usize) -> f32 {
    let distance = distance(from_x, from_y, target_x, target_y);

    1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0))
}

// Convert rotation for the map
pub fn convert_rotation(rotation: f32) -> f64 {
    // Rotate by 45'
    let rotation = rotation + 45.0f32.to_radians();

    // Convert to cartesian form
    let (x, y) = (rotation.cos(), rotation.sin());

    // Scale the y values by 0.5
    let y = y * 0.5;

    // Convert back into polar form
    let rotation = y.atan2(x);

    // Return the degrees as f64
    rotation.to_degrees() as f64
}