// Various utility functions

// Return a vec or a default if the vec is empty
macro_rules! vec_or_default {
    ($vec:expr, $default:expr) => (
        if !$vec.is_empty() {
            $vec
        } else {
            $default
        };
    )
}

// Min using partialord
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

// Max using partialord
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Linearly-interpolate between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// Ensure that a value is between lower and an upper value
pub fn clamp<T: PartialOrd>(value: T, lower: T, upper: T) -> T {
    // Calculate the smaller value
    let min = min(upper, value);
    // Calculate the bigger value
    max(min, lower)
}

// Ensure that a floating point value is between lower and upper value (as usize)
pub fn clamp_float(value: f32, lower: usize, upper: usize) -> usize {
    let value = value.round();
    let value = if value < 0.0 { 0 } else { value as usize };

    clamp(value, lower, upper)
}

// Calculate the direction between two points
pub fn direction(a_x: f32, a_y: f32, b_x: f32, b_y: f32) -> f32 {
    (b_y - a_y).atan2(b_x - a_x)
}

// Calculate the distance between two points
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
pub fn convert_rotation(mut rotation: f32) -> f32 {
    // Rotate by 45'
    rotation += 45.0_f32.to_radians();

    // Convert to cartesian form
    let (x, y) = (rotation.cos(), rotation.sin());

    // Scale the y values by 0.5 and convert back into polar form
    (y * 0.5).atan2(x)
}

#[test]
fn test_rotation() {
    // As the map is isometric, a shot fired from (0, 0) to (10, 10)
    // should be travelling directly down (90' clockwise)
    assert_eq!(convert_rotation(direction(0.0, 0.0, 10.0, 10.0)).to_degrees(), 90.0);
 
    // A shot fired from (0, 10) to (10, 0) should go going directly left
    assert_eq!(convert_rotation(direction(0.0, 10.0, 10.0, 0.0)), 0.0);
}