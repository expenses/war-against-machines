// The game's AI

use ord_subset::OrdSubsetIterExt;

use super::tiles::Visibility;
use super::map::Map;
use super::units::{Unit, UnitSide, WALK_LATERAL_COST};
use super::paths::{pathfind, PathPoint};
use super::commands::{CommandQueue, WalkCommand, FireCommand, FinishedCommand, UseItemCommand};
use utils::{chance_to_hit, distance};

// A move that the AI could take
struct AIMove {
    x: usize,
    y: usize,
    path: Vec<PathPoint>,
    cost: u16,
    target_id: Option<u8>,
    score: f32
}

impl AIMove {
    // Create a new AIMove
    fn new(x: usize, y: usize, path: Vec<PathPoint>, cost: u16, target_id: Option<u8>, score: f32) -> AIMove {
        AIMove {
            x, y, path, cost, target_id, score
        }
    }

    // Create a new AIMove set up with a specific score
    fn from(unit: &Unit, score: f32) -> AIMove {
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
        // Check if there is a closest unit and get it's ID and damage score
        let (target_id, score) = match closest_target(unit, map) {
            Some(target) => (Some(target.id), damage_score(unit.x, unit.y, 0, unit, target)),
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

    // Compare two AIMoves and take the fields from the new one if
    // it is within the units moves and has a higher score
    fn compare(&mut self, unit: &Unit, ai_move: AIMove) {
        if ai_move.score > self.score && ai_move.cost <= unit.moves {
            self.x = ai_move.x;
            self.y = ai_move.y;
            self.path = ai_move.path;
            self.cost = ai_move.cost;
            self.target_id = ai_move.target_id;
            self.score = ai_move.score;
        }
    }
}

// Attempt to make a move and return true if there are possibly more moves to make
pub fn make_move(map: &Map, command_queue: &mut CommandQueue) -> bool {
    // Get the first unit that can be moved
    if let Some(unit) = next_unit(map) {
        // If the unit has a consumable item and can use it, add a use item command
        for (index, item) in unit.inventory.iter().enumerate() {
            if unit.can_heal_from(item) || unit.can_reload_from(item) {
                command_queue.push(UseItemCommand::new(unit.id, index));
                return true;
            }
        }

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
            command_queue.push(FinishedCommand::new(unit.id));
            return true;
        }

        // If the move has a path, queue the 'walk' command to walk along it
        if !ai_move.path.is_empty() {
            command_queue.push(WalkCommand::new(unit, map, ai_move.path));
        }

        // If the move has a target, fire at the target as many times as possible
        if let Some(target) = ai_move.target_id.and_then(|id| map.units.get(id)) {
            for _ in 0 .. unit.weapon.times_can_fire(unit.moves - ai_move.cost) {
                command_queue.push(FireCommand::new(unit.id, target.x, target.y));
            }
        }

        // Return true because there are possible more moves to make
        true
    } else {
        // Return false because the AI is finished with the turn
        false
    }
}

// Find the next ai unit that can be moved
fn next_unit(map: &Map) -> Option<&Unit> {
    map.units.iter()
        // Make sure that there is a player unit alive and find ai units with avaliable moves
        .find(|unit| map.units.count(UnitSide::Player) > 0 && unit.side == UnitSide::AI && unit.moves > 0)
}

// Return an AIMove where the amount of tiles searched is maximized
fn maximize_tile_search(unit: &Unit, map: &Map) -> AIMove {
    // Create a new AIMove
    let mut ai_move = AIMove::from(unit, 0.0);

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If there is a path to the tile, check its movement score
            if let Some((path, cost)) = pathfind(unit, x, y, map) {
                ai_move.compare(unit, AIMove::new(x, y, path, cost, None, search_score(x, y, map, unit)));
            }
        }
    }

    ai_move
}

// Return an AIMove where the damage dealt to the nearest unit is maximized
fn maximize_damage(unit: &Unit, map: &Map) -> AIMove {
    // Create a new AIMove as the unit trying to fire from the current position
    let mut ai_move = AIMove::fire_from_pos(unit, map);

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If a path to the tile has been found and there is a closest target, check its damage score
            if let Some((path, cost)) = pathfind(unit, x, y, map) {
                if let Some(target) = closest_target(unit, map) {
                    let new = AIMove::new(x, y, path, cost, Some(target.id), damage_score(x, y, cost, unit, target));
                    ai_move.compare(unit, new);
                }
            }
        }
    }

    ai_move
}

// Return an AIMove where the chance_to_hit of the nearest unit is maximized
fn maximize_damage_next_turn(unit: &Unit, map: &Map) -> AIMove {
    // Find the closest target unit
    let target = closest_target(unit, map).unwrap();
    // Create a new AIMove of the chance to hit the target
    let mut ai_move = AIMove::from(unit, chance_to_hit(unit.x, unit.y, target.x, target.y));

    // Loop through the tiles
    for x in 0 .. map.tiles.cols {
        for y in 0 .. map.tiles.rows {
            // Skip the unreachable tiles
            if unreachable(unit, map, x, y) {
                continue;
            }

            // If a path to the tile has been found and there is a closest target, check its chance to hit
            if let Some((path, cost)) = pathfind(unit, x, y, map) {
                if let Some(target) = closest_target(unit, map) {
                    let new = AIMove::new(x, y, path, cost, None, chance_to_hit(x, y, target.x, target.y));
                    ai_move.compare(unit, new);
                }
            }
        }
    }

    ai_move
}

// If the tile is invisible or cannot be reached by the unit walking in a lateral direction
fn unreachable(unit: &Unit, map: &Map, x: usize, y: usize) -> bool {
    map.tiles.at(x, y).ai_visibility == Visibility::Invisible ||
    (unit.x as i32 - x as i32).abs() as u16 * WALK_LATERAL_COST > unit.moves ||
    (unit.y as i32 - y as i32).abs() as u16 * WALK_LATERAL_COST > unit.moves
}

// Find the closest target unit to a unit on the map, if any
fn closest_target<'a>(unit: &Unit, map: &'a Map) -> Option<&'a Unit> {
    map.units.iter()
        // Filter to visible player units
        .filter(|target| target.side == UnitSide::Player &&
                         map.tiles.at(target.x, target.y).ai_visibility.is_visible())
        // Minimize distance
        .ord_subset_min_by_key(|target| distance(unit.x, unit.y, target.x, target.y))
}

// Calculate the damage score for a tile
fn damage_score(x: usize, y: usize, walk_cost: u16, unit: &Unit, target: &Unit) -> f32 {
    // Return if the cost is too high
    if walk_cost > unit.moves {
        return 0.0;
    }

    // Calculate the chance to hit
    let chance_to_hit = chance_to_hit(x, y, target.x, target.y);

    // Return chance to hit * times the weapon can be fired
    chance_to_hit * f32::from(unit.weapon.times_can_fire(unit.moves - walk_cost))
}

// Calculate the search score for a tile.
// Visible tiles that were invisible count for 1.0, while visible tiles that were foggy count for 0.1
fn search_score(x: usize, y: usize, map: &Map, unit: &Unit) -> f32 {
    let mut score = 0.0;

    // Loop though the tiles
    for tile_x in 0 .. map.tiles.cols {
        for tile_y in 0 .. map.tiles.rows {
            // If the tile would be visible, add the score
            if map.tiles.line_of_sight(x, y, tile_x, tile_y, unit.tag.sight()).is_some() {
                score += match map.tiles.at(tile_x, tile_y).ai_visibility {
                    Visibility::Invisible => 1.0,
                    Visibility::Foggy => 0.1,
                    Visibility::Visible(_) => 0.0
                };
            }
        }
    }
    
    score
}

#[test]
fn test_basic_ai() {
    use super::units::UnitType;
    use super::animations::Animations;
    use super::commands::UpdateCommands;
    use ui::{TextDisplay, Vertical, Horizontal};

    let mut animations = Animations::new();
    let mut command_queue = CommandQueue::new();
    let mut log = TextDisplay::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, true);
    let mut map = Map::new(5, 5);
    map.units.add(UnitType::Squaddie, UnitSide::AI, 0, 0);
    map.units.add(UnitType::Squaddie, UnitSide::Player, 4, 4);
    map.tiles.update_visibility(&map.units);


    // On a 5x5 map, the first move that an ai unit should make is to fire onto the player unit
    assert!(make_move(&map, &mut command_queue));
    assert_eq!(command_queue.first(), Some(&FireCommand::new(0, 4, 4)));

    // After updating the command queue, there should be a bullet in animations

    command_queue.update(&mut map, &mut animations, &mut log);

    assert!(animations.first().is_some());
}