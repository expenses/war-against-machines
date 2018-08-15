// A pathfinding function and struct for the game

use pathfinding::directed::astar::astar;

use super::map::Map;
use super::units::{Unit, UnitFacing};

// Use the A Star algorithm to find a path between a unit and a destination
pub fn pathfind(unit: &Unit, dest_x: usize, dest_y: usize, map: &Map) -> Option<(Vec<PathPoint>, u16)> {
    if map.taken(dest_x, dest_y) {
        return None;
    }

    astar(
        &PathPoint::from(unit),
        |point| point.neighbours(map),
        |point| point.cost(dest_x, dest_y),
        |point| point.at(dest_x, dest_y)
    ).map(|(mut path, cost)| {
        // Remove the first point
        path.remove(0);
        (path, cost)
    })
}

// A point in the path
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PathPoint {
    pub x: usize,
    pub y: usize,
    pub cost: u16,
    pub facing: UnitFacing
}

impl PathPoint {
    // Create a new PathPoint
    pub fn new(x: usize, y: usize, cost: u16, facing: UnitFacing) -> PathPoint {
        PathPoint {
            x, y, cost, facing
        }
    }

    // Create a path point from a unit
    pub fn from(unit: &Unit) -> PathPoint {
        PathPoint {
            x: unit.x,
            y: unit.y,
            cost: 0,
            facing: unit.facing
        }
    }

    // Test if two point are at the same location (they may not have the same cost or facing)
    pub fn at(&self, x: usize, y: usize) -> bool {
        self.x == x && self.y == y
    }

    // Get the cost to a point
    fn cost(&self, x: usize, y: usize) -> u16 {
        if self.x == x || self.y == y {
            Unit::WALK_LATERAL_COST
        } else {
            Unit::WALK_DIAGONAL_COST
        }
    }

    // Get the neighbours to a point
    pub fn neighbours(&self, map: &Map) -> Vec<(PathPoint, u16)> {
        let mut neighbours = Vec::new();

        let tiles = &map.tiles;

        // lateral movement

        if self.x > 0 && tiles.horizontal_clear(self.x, self.y) {
            self.add_point(&mut neighbours, map, self.x - 1, self.y);
        }

        if self.x < tiles.width() - 1 && tiles.horizontal_clear(self.x + 1, self.y) {
            self.add_point(&mut neighbours, map, self.x + 1, self.y);
        }

        if self.y > 0 && tiles.vertical_clear(self.x, self.y) {
            self.add_point(&mut neighbours, map, self.x, self.y - 1);
        }

        if self.y < tiles.height() - 1 && tiles.vertical_clear(self.x, self.y + 1) {
            self.add_point(&mut neighbours, map, self.x, self.y + 1);
        }

        // Diagonal movement

        if tiles.diagonal_clear(self.x, self.y, true) {
            self.add_point(&mut neighbours, map, self.x - 1, self.y - 1);
        }

        if tiles.diagonal_clear(self.x + 1, self.y, false) {
            self.add_point(&mut neighbours, map, self.x + 1, self.y - 1);
        }

        if tiles.diagonal_clear(self.x, self.y + 1, false) {
            self.add_point(&mut neighbours, map, self.x - 1, self.y + 1);
        }

        if tiles.diagonal_clear(self.x + 1, self.y + 1, true) {
            self.add_point(&mut neighbours, map, self.x + 1, self.y + 1);
        }

        neighbours
    }

    // Add a point the the neighbours if it's not taken
    fn add_point(&self, neighbours: &mut Vec<(PathPoint, u16)>, map: &Map, x: usize, y: usize) {
        if !map.taken(x, y) {
            let cost = self.cost(x, y);
            let facing = UnitFacing::from_points(self.x, self.y, x, y);
            neighbours.push((PathPoint::new(x, y, cost, facing), cost));
        }
    }
}

#[test]
fn pathfinding() {
    use super::units::*;
    use super::map::*;

    let size = 30;
    let unit = Unit::new(UnitType::Squaddie, Side::PlayerA, 0, 0, UnitFacing::Bottom, 0);
    let mut map = Map::new(size, size, 0.5);

    // A path between (0, 0) and (29, 29) should be a straight diagonal

    let mut path = Vec::new();
    for i in 1 .. size {
        path.push(PathPoint::new(i, i, Unit::WALK_DIAGONAL_COST, UnitFacing::Bottom));
    }

    let cost = (size - 1) as u16 * Unit::WALK_DIAGONAL_COST;

    let path = Some((path, cost));

    assert_eq!(pathfind(&unit, size - 1, size - 1, &map), path);

    // The path should work fine if it's blocked on one side

    map.tiles.add_left_wall(1, 0, WallType::Ruin1);

    assert_eq!(pathfind(&unit, size - 1, size - 1, &map), path);

    // But not both

    map.tiles.add_top_wall(0, 1, WallType::Ruin1);

    assert_eq!(pathfind(&unit, size - 1, size - 1, &map), None);
}