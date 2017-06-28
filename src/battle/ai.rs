// The game's AI

use ord_subset::OrdSubsetIterExt;

use battle::tiles::Visibility;
use battle::map::Map;
use battle::units::{Unit, UnitSide, UNIT_SIGHT};
use battle::paths::{pathfind, PathPoint, WALK_LATERAL_COST};
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
    // Create a new AIMove set up with a specific score
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

    // Create a new AIMove as if the unit had fired from its current position at the nearest target
    fn fire_from_pos(unit: &Unit, map: &Map) -> AIMove {
        // Check if there is a closest unit and get it's id (if any) and damage score
        let (target_id, score) = match closest_target(unit, map) {
            Some((target_id, target)) => (Some(target_id), damage_score(unit.x, unit.y, 0, unit, target)),
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

    // Check some variables and compare the score with the current score of the AIMove.
    // If the new score is higher and the unit has the moves, set the variables of the AIMove.
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

// Attempt to make a move and return true if there are possibly more moves to make
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
            maximize_tile_search(unit, map)
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
            for _ in 0 .. (unit.moves - ai_move.cost) / unit.weapon.info().cost {
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

// Find the next ai unit that can be moved, if any
fn next_unit(map: &Map) -> Option<(usize, &Unit)> {
    map.units.iter()
        // Make sure that there is a player unit alive and find ai units with avaliable moves
        .find(|&(_, unit)| map.units.any_alive(UnitSide::Player) &&
                           unit.side == UnitSide::AI && unit.moves > 0)
        .map(|(unit_id, unit)| (*unit_id, unit))
}

// Return an AIMove where the amount of tiles searched is maximized
fn maximize_tile_search(unit: &Unit, map: &Map) -> AIMove {
    // Create a new AIMove
    let mut ai_move = AIMove::new(unit, 0.0);

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If there is a path to the tile, check its movement score
            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                ai_move.check_move(unit, x, y, path, cost, None, search_score(x, y, map));
            }
        }
    }

    ai_move
}

// Return an AIMove where the damage dealt to the nearest unit is maximized
fn maximize_damage(unit: &Unit, map: &Map) -> AIMove {
    // Create a new AIMove as the unit trying to fire from the current position
    let mut ai_move = AIMove::fire_from_pos(unit, &map);

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If a path to the tile has been found and there is a closest target, check its damage score
            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                if let Some((target_id, target)) = closest_target(unit, &map) {
                    ai_move.check_move(unit, x, y, path, cost, Some(target_id), damage_score(x, y, cost, unit, target));
                }
            }
        }
    }

    ai_move
}

// Return an AIMove where the chance_to_hit of the nearest unit is maximized
fn maximize_damage_next_turn(unit: &Unit, map: &Map) -> AIMove {
    // Find the closest target unit
    let (_, target) = closest_target(unit, map).unwrap();
    // Create a new AIMove of the chance to hit the target
    let mut ai_move = AIMove::new(unit, chance_to_hit(unit.x, unit.y, target.x, target.y));

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If a path to the tile has been found and there is a closest target, check its chance to hit
            if let Some((path, cost)) = pathfind(unit, x, y, &map) {
                if let Some((_, target)) = closest_target(unit, &map) {
                    ai_move.check_move(unit, x, y, path, cost, None, chance_to_hit(x, y, target.x, target.y));
                }
            }
        }
    }

    ai_move
}

// Return if the tile cannot be reached by the unit walking in a lateral direction or is invisible
fn unreachable(unit: &Unit, map: &Map, x: usize, y: usize) -> bool {
    map.tiles.at(x, y).ai_visibility == Visibility::Invisible ||
    (unit.x as i32 - x as i32).abs() as usize * WALK_LATERAL_COST > unit.moves ||
    (unit.y as i32 - y as i32).abs() as usize * WALK_LATERAL_COST > unit.moves
}

// Find the closest target unit to a unit on the map, if any
fn closest_target<'a>(unit: &Unit, map: &'a Map) -> Option<(usize, &'a Unit)> {
    map.units.iter()
        .filter(|&(_, target)| target.side == UnitSide::Player &&
                               map.tiles.at(target.x, target.y).ai_visibility == Visibility::Visible)
        .ord_subset_max_by_key(|&(_, target)| chance_to_hit(unit.x, unit.y, target.x, target.y))
        .map(|(i, unit)| (*i, unit))
}

// Calculate the damage score for a tile
fn damage_score(x: usize, y: usize, cost: usize, unit: &Unit, target: &Unit) -> f32 {
    if cost > unit.moves {
        return 0.0
    }

    let info = unit.weapon.info();
    let chance_to_hit = chance_to_hit(x, y, target.x, target.y) * info.hit_modifier;

    chance_to_hit * ((unit.moves - cost) / info.cost) as f32 * info.bullets as f32
}

// Calculate the search score for a tile.
// Visible tiles that were invisible count for 1.0, while visible tiles that were foggy count for 0.1
fn search_score(x: usize, y: usize, map: &Map) -> f32 {
    let mut score = 0.0;

    // Loop though the tiles
    for tile_x in 0 .. map.tiles.cols {
        for tile_y in 0 .. map.tiles.rows {
            // If the tile would be visible, add the score
            if distance_under(x, y, tile_x, tile_y, UNIT_SIGHT) {
                score += match map.tiles.at(tile_x, tile_y).ai_visibility {
                    Visibility::Invisible => 1.0,
                    Visibility::Foggy => 0.1,
                    Visibility::Visible => 0.0
                };
            }
        }
    }
    
    score
}