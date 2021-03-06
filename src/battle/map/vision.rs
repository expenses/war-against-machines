// The line of sight tile functions are in here, because they're pretty big and kinda seperate from the other stuff

use super::super::units::*;
use super::tiles::*;
use super::walls::*;

use line_drawing::*;
use rand;
use std::mem::*;

// A point for line-of-sight
type Point = (isize, isize);

// Sort two points on the y axis
fn sort(a: Point, b: Point) -> (Point, Point, bool) {
    if a.1 > b.1 {
        (b, a, true)
    } else {
        (a, b, false)
    }
}

// Convert a coord in `usize`s to a point
fn to_point(x: usize, y: usize) -> Point {
    (x as isize, y as isize)
}

// Convert a point back into `usize`s
fn from_point(point: Point) -> (usize, usize) {
    (point.0 as usize, point.1 as usize)
}

// Return whether there is a wall between two tiles
impl Tiles {
    fn wall_between(&self, a: Point, b: Point) -> bool {
        let ((a_x, a_y), (b_x, b_y)) = (from_point(a), from_point(b));

        !match (b.0 - a.0, b.1 - a.1) {
            (0, 1) => self.vertical_clear(b_x, b_y),
            (1, 0) => self.horizontal_clear(b_x, b_y),
            (-1, 0) => self.horizontal_clear(a_x, a_y),
            (-1, 1) => self.diagonal_clear(a_x, b_y, false),
            (1, 1) => self.diagonal_clear(b_x, b_y, true),
            _ => unreachable!(),
        }
    }

    // Return the first blocking obstacle between two points or none
    pub fn line_of_fire(
        &self,
        start_x: usize,
        start_y: usize,
        end_x: usize,
        end_y: usize,
    ) -> Option<(Point, WallSide)> {
        // Convert the points to isize and sort
        let (start, end) = (to_point(start_x, start_y), to_point(end_x, end_y));
        let (start, end, reversed) = sort(start, end);

        // Create an iterator of tile steps
        let mut iter = Bresenham::new(start, end)
            .steps()
            // Filter to steps with walls between
            .filter(|&(a, b)| self.wall_between(a, b))
            // Map to the containing tile and wall direction
            .map(|(a, b)| match (b.0 - a.0, b.1 - a.1) {
                (0, 1) => (b, WallSide::Top),
                (1, 0) => (b, WallSide::Left),
                (-1, 0) => (a, WallSide::Left),
                // For diagonal steps we have to randomly pick one of the two closest walls
                (-1, 1) | (1, 1) => {
                    let left_to_right = a.0 < b.0;

                    // Get the four walls segments between the tiles if left-to-right or their flipped equivalents
                    let (mut top, mut left, mut right, mut bottom) = if left_to_right {
                        ((b.0, a.1), (a.0, b.1), b, b)
                    } else {
                        (a, (a.0, b.1), b, (a.0, b.1))
                    };

                    // Swap the points around if the line is reversed
                    if reversed {
                        swap(&mut top, &mut bottom);
                        swap(&mut left, &mut right);
                    }

                    // Get whether each of these segments contain walls
                    let top_block = !self.horizontal_clear(top.0 as usize, top.1 as usize);
                    let left_block = !self.vertical_clear(left.0 as usize, left.1 as usize);
                    let right_block = !self.vertical_clear(right.0 as usize, right.1 as usize);
                    let bottom_block = !self.horizontal_clear(bottom.0 as usize, bottom.1 as usize);

                    // Get the pairs of walls to choose from
                    let (wall_a, wall_b) = if top_block && left_block {
                        ((top, WallSide::Left), (left, WallSide::Top))
                    } else if left_block && right_block {
                        ((left, WallSide::Top), (right, WallSide::Top))
                    } else if top_block && bottom_block {
                        ((top, WallSide::Left), (bottom, WallSide::Left))
                    } else {
                        ((bottom, WallSide::Left), (right, WallSide::Top))
                    };

                    // Choose a random wall
                    if rand::random::<bool>() {
                        wall_a
                    } else {
                        wall_b
                    }
                }
                _ => unreachable!(),
            });

        // Return either the last or first wall found or none
        if reversed {
            iter.last()
        } else {
            iter.next()
        }
    }

    // Would a unit with a particular sight range be able to see from one tile to another
    // Return the number of tiles away a point is, or none if visibility is blocked
    pub fn line_of_sight(
        &self,
        a_x: usize,
        a_y: usize,
        b_x: usize,
        b_y: usize,
        sight: f32,
        facing: UnitFacing,
    ) -> Option<u8> {
        if facing.can_see(a_x, a_y, b_x, b_y, sight) {
            // Sort the points so that line-of-sight is symmetrical
            let (start, end, _) = sort(to_point(a_x, a_y), to_point(b_x, b_y));
            let mut distance = 0;

            for (a, b) in Bresenham::new(start, end).steps() {
                // Return if line of sight is blocked by a wall
                if self.wall_between(a, b) {
                    return None;
                }

                // Increase the distance
                distance += if a.0 == b.0 || a.1 == b.1 {
                    Unit::WALK_LATERAL_COST
                } else {
                    Unit::WALK_DIAGONAL_COST
                } as u8;
            }

            Some(distance)
        } else {
            None
        }
    }
}

#[test]
fn unit_visibility() {
    use super::super::paths::*;
    use super::super::units::*;

    let mut tiles = Tiles::new(30, 30);
    let mut units = Units::new();
    units.add(UnitType::Squaddie, Side::PlayerA, 0, 0, UnitFacing::Bottom);
    tiles.update_visibility(&units);

    // A tile a unit is standing on should be visible with a distance of 0
    assert_eq!(
        tiles.visibility_at(0, 0, Side::PlayerA),
        Visibility::Visible(0)
    );
    // A far away tile should be invisible
    assert_eq!(
        tiles.visibility_at(29, 29, Side::PlayerA),
        Visibility::Invisible
    );

    // A tile that was visible but is no longer should be foggy

    units
        .get_mut(0)
        .unwrap()
        .move_to(&PathPoint::new(29, 0, 0, UnitFacing::Top));
    tiles.update_visibility(&units);

    assert_eq!(tiles.visibility_at(0, 0, Side::PlayerA), Visibility::Foggy);

    // If the unit is boxed into a corner, only it's tile should be visible

    tiles.add_left_wall(29, 0, WallType::Ruin1);
    tiles.add_top_wall(29, 1, WallType::Ruin2);

    tiles.update_visibility(&units);

    for (x, y) in tiles.iter() {
        let visibility = tiles.visibility_at(x, y, Side::PlayerA);

        if x == 29 && y == 0 {
            assert_eq!(visibility, Visibility::Visible(0));
            assert!(!tiles.visibility_at(x, y, Side::PlayerA).is_invisible());
        } else {
            assert!(!visibility.is_visible());
        }
    }
}

#[test]
fn line_of_fire() {
    let mut tiles = Tiles::new(5, 5);

    tiles.add_left_wall(1, 0, WallType::Ruin1);
    tiles.add_top_wall(0, 1, WallType::Ruin1);
    tiles.add_top_wall(1, 1, WallType::Ruin1);
    tiles.add_left_wall(1, 1, WallType::Ruin1);

    let top = Some(((1, 0), WallSide::Left));
    let left = Some(((0, 1), WallSide::Top));
    let right = Some(((1, 1), WallSide::Top));
    let bottom = Some(((1, 1), WallSide::Left));

    // Test lateral directions

    assert_eq!(tiles.line_of_fire(0, 0, 1, 0), top);
    assert_eq!(tiles.line_of_fire(0, 0, 0, 1), left);

    // Test diagonal directions

    let diag_1 = tiles.line_of_fire(0, 0, 1, 1);
    assert!(diag_1 == top || diag_1 == left);
    let diag_2 = tiles.line_of_fire(1, 1, 0, 0);
    assert!(diag_2 == bottom || diag_2 == right);
    let diag_3 = tiles.line_of_fire(0, 1, 1, 0);
    assert!(diag_3 == left || diag_3 == bottom);
    let diag_4 = tiles.line_of_fire(1, 0, 0, 1);
    assert!(diag_4 == right || diag_4 == top);
}
