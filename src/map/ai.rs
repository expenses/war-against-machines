use ord_subset::OrdSubsetIterExt;

use map::map::Map;
use map::units::{Unit, UnitSide};
use map::paths::{pathfind, PathPoint};
use utils::{distance, chance_to_hit};

// A move that the AI could take
struct AIMove {
    x: usize,
    y: usize,
    path: Vec<PathPoint>,
    cost: usize,
    score: f32
}

impl AIMove {
    fn from(unit: &Unit, target: &Unit) -> AIMove {
        AIMove {
            x: unit.x,
            y: unit.y,
            path: Vec::new(),
            cost: 0,
            score: tile_score(unit.x, unit.y, 0, unit, target)
        }
    }

    fn check_move(&mut self, x: usize, y: usize, path: Vec<PathPoint>, cost: usize, score: f32) {
        if score > self.score {
            self.x = x;
            self.y = y;
            self.path = path;
            self.cost = cost;
            self.score = score;
        }
    }
}

pub fn take_turn(mut map: &mut Map) {
    for unit_id in 0 .. map.units.len() {
        if skip(unit_id, map) {
            continue;
        }

        let target_id = closest_target(unit_id, map);

        match target_id {
            Some(target_id) => {
                let ai_move = {
                    let unit = map.units.get(unit_id);
                    let target = map.units.get(target_id);
                    let mut ai_move = AIMove::from(unit, target);

                    for x in 0 .. map.tiles.cols {
                        for y in 0 .. map.tiles.rows {
                            match pathfind(unit, x, y, &map) {
                                Some((path, cost)) => {
                                    let score = tile_score(x, y, cost, unit, target);

                                    ai_move.check_move(x, y, path, cost, score);
                                }
                                _ => {}
                            }
                        }
                    }

                    ai_move
                };

                let (unit, target) = map.units.get_two_mut(unit_id, target_id);

                unit.move_to(unit_id, ai_move.path, ai_move.cost, &mut map.animation_queue);

                for _ in 0 .. unit.moves / unit.weapon.cost {
                    unit.fire_at(target_id, target, &mut map.animation_queue);
                }
            }
            _ => {}
        }
    }
}

fn skip(unit_id: usize, map: &Map) -> bool {
    let unit = map.units.get(unit_id);

    unit.side != UnitSide::Enemy ||
    !unit.alive() ||
    !map.units.any_alive(UnitSide::Friendly)
}

fn closest_target(unit_id: usize, map: &Map) -> Option<usize> {
    let unit = map.units.get(unit_id);

    map.units.iter()
        .enumerate()
        .filter(|&(_, unit)| unit.side == UnitSide::Friendly && unit.alive())
        .ord_subset_min_by_key(|&(_, target)| {
            distance(unit.x, unit.y, target.x, target.y)
        })
        .and_then(|(i, _)| Some(i))
}

fn tile_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    chance_to_hit(x, y, target.x, target.y) * ((unit.moves - cost) / unit.weapon.cost) as f32
}
