// Various utility functions

// Ensure that a value is between lower and an upper value
macro_rules! clamp {
    ($value:expr, $lower:expr, $upper:expr) => ({
        // Calculate the smaller value
        let min = if $upper > $value {$value} else {$upper};
        // Calculate the bigger value
        if min > $lower {min} else {$lower}
    })
}

// Ensure that a floating point value is between lower and upper value (as usize)
pub fn clamp_float(value: f64, lower: usize, upper: usize) -> usize {
    let value = value.round();
    let value = if value < 0.0 { 0 } else { value as usize };

    clamp!(value, lower, upper)
}

// Calculate the distance between two points on the map
pub fn distance(a_x: usize, a_y: usize, b_x: usize, b_y: usize) -> f32 {
    (a_x as f32 - b_x as f32).hypot(a_y as f32 - b_y as f32)
}

// Calculate if a distance between two points is below a given value
pub fn distance_under(a_x: usize, a_y: usize, b_x: usize, b_y: usize, value: f32) -> bool {
    // First test the linear distances
    (a_x as f32 - b_x as f32).abs() <= value &&
    (a_y as f32 - b_y as f32).abs() <= value &&
    // Then the hypot distance
    distance(a_x, a_y, b_x, b_y) <= value
}

// A chance-to-hit function based on a fairly simple sigmoid curve.
pub fn chance_to_hit(a_x: usize, a_y: usize, b_x: usize, b_y: usize) -> f32 {
    let distance = distance(a_x, a_y, b_x, b_y);

    (1.0 / (1.0 + 0.02 * 4.0_f32.powf(distance / 3.0)))
}

// Convert a rotation for drawing on the map
pub fn convert_rotation(rotation: f64) -> f64 {
    // Rotate by 45'
    let rotation = rotation + 45.0_f64.to_radians();

    // Convert to cartesian form
    let (x, y) = (rotation.cos(), rotation.sin());

    // Scale the y values by 0.5
    let y = y * 0.5;

    // Convert back into polar form
    y.atan2(x)
}