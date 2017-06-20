use ord_subset::OrdSubsetIterExt;

use map::map::Map;
use map::units::{Unit, UnitSide};
use map::paths::{pathfind, PathPoint, WALK_STRAIGHT_COST};
use map::commands::{WalkCommand, FireCommand};
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
        let target_id = closest_target(unit, map);
        let target = map.units.get(target_id);

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
    for unit_id in 0 .. map.units.len() {
        if skip(unit_id, map) {
            continue;
        }

        let ai_move = {
            let unit = map.units.get(unit_id);
            let mut ai_move = AIMove::from(unit, &map);

            for x in 0 .. map.tiles.cols {
                for y in 0 .. map.tiles.rows {
                    if unreachable(unit, x, y) {
                        continue;
                    }

                    match pathfind(unit, x, y, &map) {
                        Some((path, cost)) => {
                            let target_id = closest_target(unit, &map);
                            let score = tile_score(x, y, cost, unit, map.units.get(target_id));

                            ai_move.check_move(x, y, path, cost, target_id, score);
                        }
                        _ => {}
                    }
                }
            }

            ai_move
        };

        let unit = map.units.get(unit_id);

        if ai_move.path.len() > 0 {
            map.command_queue.add_walk(WalkCommand::new(unit_id, ai_move.path));
        }

        for _ in 0 .. (unit.moves - ai_move.cost) / unit.weapon.cost {
            map.command_queue.add_fire(FireCommand::new(unit_id, ai_move.target_id));
        }
    }
}

fn skip(unit_id: usize, map: &Map) -> bool {
    let unit = map.units.get(unit_id);

    unit.side != UnitSide::Enemy ||
    !unit.alive() ||
    !map.units.any_alive(UnitSide::Friendly)
}

fn unreachable(unit: &Unit, x: usize, y: usize) -> bool {
    (unit.x as i32 - x as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves ||
    (unit.y as i32 - y as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves
}

fn closest_target(unit: &Unit, map: &Map) -> usize {
    map.units.iter()
        .enumerate()
        .filter(|&(_, target)| target.side == UnitSide::Friendly && target.alive())
        .ord_subset_min_by_key(|&(_, target)| distance(unit.x, unit.y, target.x, target.y))
        .map(|(i, _)| i)
        .unwrap()
}

fn tile_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    chance_to_hit(x, y, target.x, target.y) * ((unit.moves - cost) / unit.weapon.cost) as f32
}
