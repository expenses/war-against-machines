use pathfinding;

use std::cmp::{min, max};

use map::map::Map;
use units::Unit;

const WALK_STRAIGHT_COST: usize = 2;
const WALK_DIAGONAL_COST: usize = 3;

// Use the A Star algorithm to find a path between a unit and a destination
pub fn pathfind(unit: &Unit, dest: &PathPoint, map: &Map) -> Option<(Vec<PathPoint>, usize)> {
    pathfinding::astar(
        &PathPoint::from(unit),
        |point| point.neighbours(map),
        |point| point.cost(dest),
        |point| point.at(dest)
    ).and_then(|(mut path, cost)| {
        // Remove the first point
        path.remove(0);
        Some((path, cost))
    })
}

// A point in the path
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PathPoint {
    pub x: usize,
    pub y: usize,
    pub cost: usize
}

impl PathPoint {
    // Create a new path point
    pub fn new(x: usize, y: usize) -> PathPoint {
        PathPoint {
            x, y,
            cost: 0
        }
    }

    // Create a path point from a unit
    fn from(unit: &Unit) -> PathPoint {
        PathPoint {
            x: unit.x,
            y: unit.y,
            cost: 0
        }
    }

    // Test if two point are at the sme location (they may not have the same cost)
    pub fn at(&self, point: &PathPoint) -> bool {
        self.x == point.x && self.y == point.y
    }

    // Get the cost to a past
    fn cost(&self, point: &PathPoint) -> usize {
        if self.x == point.x || self.y == point.y {
            WALK_STRAIGHT_COST
        } else {
            WALK_DIAGONAL_COST
        }
    }

    fn neighbours(&self, map: &Map) -> Vec<(PathPoint, usize)> {
        let mut neighbours = Vec::new();

        let min_x = max(0, self.x as i32 - 1) as usize;
        let min_y = max(0, self.y as i32 - 1) as usize;

        let max_x = min(map.tiles.cols - 1, self.x + 1);
        let max_y = min(map.tiles.rows - 1, self.y + 1);

        // Loop through the possible neighbours
        for x in min_x .. max_x + 1 {
            for y in min_y .. max_y + 1 {
                if !map.taken(x, y) && !(x == self.x && y == self.y) {
                    // Add the point

                    let mut point = PathPoint::new(x, y);
                    let cost = self.cost(&point);
                    point.cost = self.cost + cost;

                    neighbours.push((point, cost));
                }
            }
        }

        return neighbours;
    }
}