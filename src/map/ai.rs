use ord_subset::OrdSubsetIterExt;

use map::map::Map;
use map::units::{Unit, UnitSide};
use map::paths::{pathfind, PathPoint, WALK_STRAIGHT_COST};
use map::commands::{Command, WalkCommand, FireCommand};
use utils::{distance, chance_to_hit};

// A move that the AI could take
struct AIMove {
    x: usize,
    y: usize,
    path: Vec<PathPoint>,
    cost: usize,
    target_id: usize,
    score: f32
}

impl AIMove {
    fn from(unit: &Unit, map: &Map) -> AIMove {
        let (target_id, target) = closest_target(unit, map);

        AIMove {
            x: unit.x,
            y: unit.y,
            path: Vec::new(),
            cost: 0,
            target_id,
            score: tile_score(unit.x, unit.y, 0, unit, target)
        }
    }

    fn check_move(&mut self, x: usize, y: usize, path: Vec<PathPoint>, cost: usize, target_id: usize, score: f32) {
        if score > self.score {
            self.x = x;
            self.y = y;
            self.path = path;
            self.cost = cost;
            self.target_id = target_id;
            self.score = score;
        }
    }
}

pub fn take_turn(mut map: &mut Map) {
    if !map.units.any_alive(UnitSide::Friendly) {
        return;
    }

    for (unit_id, unit) in map.units.iter()
        .filter(|&(_, unit)| unit.side == UnitSide::Enemy)
        .map(|(i, unit)| (*i, unit)) {
        
        let mut ai_move = AIMove::from(unit, &map);

        for x in 0 .. map.tiles.cols {
            for y in 0 .. map.tiles.rows {
                if unreachable(unit, x, y) {
                    continue;
                }

                match pathfind(unit, x, y, &map) {
                    Some((path, cost)) => {
                        let (target_id, target) = closest_target(unit, &map);
                        let score = tile_score(x, y, cost, unit, target);

                        ai_move.check_move(x, y, path, cost, target_id, score);
                    }
                    _ => {}
                }
            }
        }

        if ai_move.path.len() > 0 {
            map.command_queue.push(Command::Walk(WalkCommand::new(unit_id, ai_move.path)));
        }

        for _ in 0 .. (unit.moves - ai_move.cost) / unit.weapon.cost {
            map.command_queue.push(Command::Fire(FireCommand::new(unit_id, ai_move.target_id)));
        }
    }
}

// If the tile cannot be reached by the unit walking in a linear direction
fn unreachable(unit: &Unit, x: usize, y: usize) -> bool {
    (unit.x as i32 - x as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves ||
    (unit.y as i32 - y as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves
}

// Find the closest target unit to unit on the map
fn closest_target<'a>(unit: &Unit, map: &'a Map) -> (usize, &'a Unit) {
    map.units.iter()
        .filter(|&(_, target)| target.side == UnitSide::Friendly)
        .ord_subset_min_by_key(|&(_, target)| distance(unit.x, unit.y, target.x, target.y))
        .map(|(i, unit)| (*i, unit))
        .unwrap()
}

// Calculate the score for a tile
fn tile_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    chance_to_hit(x, y, target.x, target.y) * ((unit.moves - cost) / unit.weapon.cost) as f32
}
