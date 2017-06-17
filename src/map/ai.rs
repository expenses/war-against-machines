use ord_subset::OrdSubsetIterExt;

use map::map::Map;
use map::units::{Unit, UnitSide};
use map::paths::pathfind;
use utils::{distance, chance_to_hit};

// A move that the AI could take
struct AIMove {
    x: usize,
    y: usize,
    cost: usize,
    score: f32
}

pub fn take_turn(map: &mut Map) {
    for id in 0 .. map.units.len() {
        if map.units.get(id).side != UnitSide::Enemy || !map.units.get(id).alive() {
            continue;
        }

        let target_id = map.units.iter()
            .enumerate()
            .filter(|&(_, unit)| unit.side == UnitSide::Friendly && unit.alive())
            .ord_subset_min_by_key(|&(_, target)| {
                let unit = map.units.get(id);
                distance(unit.x, unit.y, target.x, target.y)
            })
            .and_then(|(i, _)| Some(i));

        match target_id {
            Some(target_id) => {
                let ai_move = {
                    let unit = map.units.get(id);
                    let target = map.units.get(target_id);
                    let mut ai_move = AIMove {
                        x: unit.x,
                        y: unit.y,
                        cost: 0,
                        score: tile_score(unit.x, unit.y, 0, unit, target)
                    };

                    for x in 0 .. map.tiles.cols {
                        for y in 0 .. map.tiles.rows {
                            match pathfind(unit, x, y, &map) {
                                Some((_, cost)) => {
                                    let score = tile_score(x, y, cost, unit, target);

                                    if score > ai_move.score {
                                        println!("Enemy {} found ({}, {}) with a score of {}", id, x, y, score);
                                        ai_move = AIMove {x, y, cost, score};
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    ai_move
                };

                let (unit, target) = map.units.get_two_mut(id, target_id);

                if unit.move_to(ai_move.x, ai_move.y, ai_move.cost) {
                    for _ in 0 .. unit.moves / unit.weapon.cost {
                        unit.fire_at(target, &mut map.animation_queue);
                    }
                }
            }
            _ => {}
        }
    }
}

fn tile_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    chance_to_hit(x, y, target.x, target.y) * ((unit.moves - cost) / unit.weapon.cost) as f32
}
