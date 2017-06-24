use ord_subset::OrdSubsetIterExt;

use battle::tiles::Visibility;
use battle::map::Map;
use battle::units::{Unit, UnitSide};
use battle::tiles::UNIT_SIGHT;
use battle::paths::{pathfind, PathPoint, WALK_STRAIGHT_COST};
use battle::commands::{CommandQueue, Command, WalkCommand, FireCommand, FinishedCommand};
use utils::{chance_to_hit, distance_under};

// A move that the AI could take
#[derive(Debug)]
struct AIMove {
    x: usize,
    y: usize,
    path: Vec<PathPoint>,
    cost: usize,
    target_id: Option<usize>,
    score: f32
}

impl AIMove {
    fn new(unit: &Unit, score: f32) -> AIMove {
        AIMove {
            x: unit.x,
            y: unit.y,
            path: Vec::new(),
            cost: 0,
            target_id: None,
            score
        }
    }

    fn fire_from_pos(unit: &Unit, map: &Map) -> AIMove {
        let (target_id, score) = match closest_target(unit, map) {
            Some((target_id, target)) => (Some(target_id), firing_score(unit.x, unit.y, 0, unit, target)),
            None => (None, 0.0)
        };

        AIMove {
            x: unit.x,
            y: unit.y,
            path: Vec::new(),
            cost: 0,
            target_id,
            score
        }
    }

    fn check_move(&mut self, unit: &Unit, x: usize, y: usize, path: Vec<PathPoint>, cost: usize, target_id: Option<usize>, score: f32) {
        if score > self.score && cost <= unit.moves {
            self.x = x;
            self.y = y;
            self.path = path;
            self.cost = cost;
            self.target_id = target_id;
            self.score = score;
        }
    }
}

pub fn make_move(map: &Map, command_queue: &mut CommandQueue) -> bool {
    // Get the first unit that can be moved
    if let Some((unit_id, unit)) = next_unit(map) {
        // Determine if any targets are visible
        let visible_target = closest_target(unit, map).is_some();

        let ai_move = if visible_target {
            // If a target is visible, attempt to maximize damage
            let ai_move = maximize_damage(unit, map);

            if ai_move.score > 0.25 {
                ai_move
            } else {
                // If the score is too low, attempt to find a point where damage would be maximized next turn
                maximize_damage_next_turn(unit, map)
            }
        } else {
            // Otherwise, find the most new tiles
            maximize_tile_discovery(unit, map)
        };

        // If the move doesn't have a cost or a target, queue the 'finished' command to set the units moves to 0
        if ai_move.cost == 0 && ai_move.target_id.is_none() {
            command_queue.push(Command::Finished(FinishedCommand::new(unit_id)));
            return true;
        }

        // If the move has a path, queue the 'walk' command to walk along it
        if ai_move.path.len() > 0 {
            command_queue.push(Command::Walk(WalkCommand::new(unit_id, map, ai_move.path)));
        }

        // If the move has a target, fire at the target as many times as possible
        if let Some(target_id) = ai_move.target_id {
            for _ in 0 .. (unit.moves - ai_move.cost) / unit.weapon.cost {
                command_queue.push(Command::Fire(FireCommand::new(unit_id, target_id)));
            }
        }

        // Return true because there are possible more moves to make
        true
    } else {
        // Return false because the AI is finished with the turn
        false
    }
}

fn next_unit(map: &Map) -> Option<(usize, &Unit)> {
    map.units.iter()
        .find(|&(_, unit)| map.units.any_alive(UnitSide::Friendly) &&
                           unit.side == UnitSide::Enemy && unit.moves > 0)
        .map(|(unit_id, unit)| (*unit_id, unit))
}

fn maximize_tile_discovery(unit: &Unit, map: &Map) -> AIMove {
    let mut ai_move = AIMove::new(unit, 0.0);

    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            if unreachable(unit, map, x, y) {
                continue;
            }

            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                ai_move.check_move(unit, x, y, path, cost, None, movement_score(x, y, map));
            }
        }
    }

    ai_move
}

fn maximize_damage(unit: &Unit, map: &Map) -> AIMove {
    let mut ai_move = AIMove::fire_from_pos(unit, &map);

    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            if unreachable(unit, map, x, y) {
                continue;
            }

            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                if let Some((target_id, target)) = closest_target(unit, &map) {
                    ai_move.check_move(unit, x, y, path, cost, Some(target_id), firing_score(x, y, cost, unit, target));
                }
            }
        }
    }

    ai_move
}

fn maximize_damage_next_turn(unit: &Unit, map: &Map) -> AIMove {
    let (_, target) = closest_target(unit, map).unwrap();
    let mut ai_move = AIMove::new(unit, chance_to_hit(unit.x, unit.y, target.x, target.y));

    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            if unreachable(unit, map, x, y) {
                continue;
            }

            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                if let Some((_, target)) = closest_target(unit, &map) {
                    ai_move.check_move(unit, x, y, path, cost, None, chance_to_hit(x, y, target.x, target.y));
                }
            }
        }
    }

    ai_move
}

// If the tile cannot be reached by the unit walking in a linear direction
fn unreachable(unit: &Unit, map: &Map, x: usize, y: usize) -> bool {
    map.tiles.at(x, y).enemy_visibility == Visibility::Invisible ||
    (unit.x as i32 - x as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves ||
    (unit.y as i32 - y as i32).abs() as usize * WALK_STRAIGHT_COST > unit.moves
}

// Find the closest target unit to unit on the map
fn closest_target<'a>(unit: &Unit, map: &'a Map) -> Option<(usize, &'a Unit)> {
    map.units.iter()
        .filter(|&(_, target)| target.side == UnitSide::Friendly &&
                               map.tiles.at(target.x, target.y).enemy_visibility == Visibility::Visible)
        .ord_subset_max_by_key(|&(_, target)| chance_to_hit(unit.x, unit.y, target.x, target.y))
        .map(|(i, unit)| (*i, unit))
}

// Calculate the score for a tile
fn firing_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    chance_to_hit(x, y, target.x, target.y) * ((unit.moves - cost) / unit.weapon.cost) as f32
}

fn movement_score(x: usize, y: usize, map: &Map) -> f32 {
    let mut score = 0.0;

    for tile_x in 0 .. map.tiles.cols {
        for tile_y in 0 .. map.tiles.rows {
            if distance_under(x, y, tile_x, tile_y, UNIT_SIGHT) {
                score += match map.tiles.at(tile_x, tile_y).enemy_visibility {
                    Visibility::Invisible => 1.0,
                    Visibility::Foggy => 0.1,
                    Visibility::Visible => 0.0
                };
            }
        }
    }
    
    score
}