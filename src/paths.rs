use pathfinding;

use std::cmp::{min, max};

use map::Map;
use units::Squaddie;

const WALK_COST: usize = 2;
const WALK_DIAGONAL_COST: usize = 3;

pub fn pathfind(start: &PathPoint, end: &PathPoint, map: &Map) -> Option<(Vec<PathPoint>, usize)> {
    pathfinding::astar(
        &start,
        |point| point.neighbours(map),
        |point| point.cost(&end),
        |point| *point == *end
    )
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PathPoint {
    pub x: usize,
    pub y: usize
}

impl PathPoint {
    pub fn new(x: usize, y: usize) -> PathPoint {
        PathPoint {
            x, y
        }
    }

    pub fn from(squaddie: &Squaddie) -> PathPoint {
        PathPoint {
            x: squaddie.x,
            y: squaddie.y
        }
    }

    fn cost(&self, point: &PathPoint) -> usize {
        if self.x == point.x || self.y == point.y { WALK_COST } else { WALK_DIAGONAL_COST }
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
                    let point = PathPoint::new(x, y);
                    let cost = self.cost(&point);
                    neighbours.push((point, cost));
                }
            }
        }

        return neighbours;
    }
}