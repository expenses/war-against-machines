use pathfinding;

use std::cmp::{min, max};

use map::Map;
use units::Unit;

const WALK_STRAIGHT_COST: usize = 2;
const WALK_DIAGONAL_COST: usize = 3;

pub fn pathfind(start: &PathPoint, end: &PathPoint, map: &Map) -> Option<(Vec<PathPoint>, usize)> {
    pathfinding::astar(
        start,
        |point| point.neighbours(map),
        |point| point.cost(end),
        |point| point.at(end)
    ).and_then(|(mut path, cost)| {
        path.remove(0);
        Some((path, cost))
    })
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PathPoint {
    pub x: usize,
    pub y: usize,
    pub cost: usize
}

impl PathPoint {
    pub fn new(x: usize, y: usize) -> PathPoint {
        PathPoint {
            x, y,
            cost: 0
        }
    }

    pub fn from(unit: &Unit) -> PathPoint {
        PathPoint {
            x: unit.x,
            y: unit.y,
            cost: 0
        }
    }

    pub fn at(&self, point: &PathPoint) -> bool {
        self.x == point.x && self.y == point.y
    }

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

        for x in min_x .. max_x + 1 {
            for y in min_y .. max_y + 1 {
                if !map.taken(x, y) {
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