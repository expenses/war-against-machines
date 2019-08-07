use super::map::*;
use super::networking::*;
use super::paths::*;
use super::units::*;
use error::*;
use ord_subset::*;
use utils::*;

use std::collections::*;
use std::thread::sleep;
use std::time::*;

#[derive(Debug)]
enum AIMove<'a> {
    Walk(Vec<PathPoint>),
    Fire(&'a Unit),
    None,
}

impl<'a> AIMove<'a> {
    fn new_walk(path: Vec<PathPoint>) -> Self {
        if !path.is_empty() {
            AIMove::Walk(path)
        } else {
            AIMove::None
        }
    }
}

struct Walk {
    path: Vec<PathPoint>,
    score: f32,
}

impl Walk {
    fn new(path: Vec<PathPoint>, score: f32) -> Self {
        Self { path, score }
    }

    fn update(&mut self, other: Walk) {
        if other.score > self.score {
            *self = other;
        }
    }
}

pub struct AIClient {
    client: Client,
    finished_units: HashSet<u8>,
    waiting_for_response: Option<u8>,
}

impl AIClient {
    pub fn new(connection: ClientConn) -> Result<Self> {
        Ok(Self {
            client: Client::new(connection)?,
            finished_units: HashSet::new(),
            waiting_for_response: None,
        })
    }

    fn map(&self) -> &Map {
        &self.client.map
    }

    // Find the closest target unit to a unit on the map, if any
    fn closest_target(&self, unit: &Unit) -> Option<&Unit> {
        self.map()
            .units
            .iter()
            // Filter to visible player units
            .filter(|target| target.side != self.client.side)
            // Minimize distance
            .ord_subset_min_by_key(|target| distance(unit.x, unit.y, target.x, target.y))
    }

    // Iterate over tiles a unit could reach
    fn reachable_tiles<'a>(&'a self, unit: &'a Unit) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.map().tiles.iter().filter(move |&(x, y)| {
            !self.client.visibility_at(x, y).is_invisible()
                && (unit.x as i32 - x as i32).abs() as u16 * Unit::WALK_LATERAL_COST <= unit.moves
                && (unit.y as i32 - y as i32).abs() as u16 * Unit::WALK_LATERAL_COST <= unit.moves
        })
    }

    fn pathfind(&self, unit: &Unit, x: usize, y: usize) -> Option<(Vec<PathPoint>, u16)> {
        pathfind(unit, x, y, self.map()).filter(|(_, cost)| *cost <= unit.moves)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let message_recieved = self.client.recv();
            let (finished, invalid_command) = self.client.process_state_updates();

            if invalid_command {
                if let Some(id) = self.waiting_for_response.take() {
                    debug!("Invalid command issued by {}", id);
                    self.finished_units.insert(id);
                }
            }

            if message_recieved {
                self.waiting_for_response = None;
            }

            if finished {
                return Ok(());
            }

            if self.waiting_for_response.is_none() && self.client.our_turn() {
                let next_unit = self.client.map.units.iter().find(|unit| {
                    unit.side == self.client.side
                        && unit.moves > 0
                        && !self.finished_units.contains(&unit.id)
                });

                if let Some(unit) = next_unit {
                    let sent_message = self.process_unit(unit);
                    if sent_message {
                        self.waiting_for_response = Some(unit.id);
                    } else {
                        self.finished_units.insert(unit.id);
                    }
                } else {
                    self.client.end_turn();
                    self.finished_units.clear();
                }
            }

            sleep(Duration::from_millis(1));
        }
    }

    fn process_unit(&self, unit: &Unit) -> bool {
        for (index, item) in unit.inventory().iter().enumerate() {
            if unit.can_heal_from(*item) || unit.can_reload_from(*item) {
                self.client.use_item(unit.id, index);
                return true;
            }
        }

        let ai_move = match self.closest_target(unit) {
            Some(target) => {
                if !unit.weapon.can_fire() {
                    AIMove::None
                } else if self.damage_score(unit.x, unit.y, 0, unit, target) < 50.0 {
                    self.maximise_damage(unit, target)
                } else {
                    AIMove::Fire(target)
                }
            }
            None => self.maximize_tile_search(unit),
        };

        debug!("Move: {:?}", ai_move);
        debug!("Finished: {:?}", self.finished_units);

        match ai_move {
            AIMove::Fire(target) => {
                self.client.fire(unit.id, target.x, target.y);
                true
            }
            AIMove::Walk(path) => {
                self.client.walk(unit.id, &path);
                true
            }
            AIMove::None => false,
        }
    }

    // Return an path where the chance_to_hit of the nearest unit is maximized
    fn maximise_damage(&self, unit: &Unit, target: &Unit) -> AIMove {
        // Create a new AIMove of the chance to hit the target
        let mut walk = Walk::new(
            Vec::new(),
            self.damage_score(unit.x, unit.y, 0, unit, target),
        );

        // Loop through the reachable tiles
        for (x, y) in self.reachable_tiles(unit) {
            // If a path to the tile has been found and there is a closest target, check its chance to hit
            if let Some((path, cost)) = self.pathfind(unit, x, y) {
                if let Some(target) = self.closest_target(unit) {
                    walk.update(Walk::new(path, self.damage_score(x, y, cost, unit, target)));
                }
            }
        }

        AIMove::new_walk(walk.path)
    }

    // Return an path where the tiles searched is maximized
    fn maximize_tile_search(&self, unit: &Unit) -> AIMove {
        let mut walk = Walk::new(Vec::new(), 0.0);

        // Loop through the reachable tiles
        for (x, y) in self.reachable_tiles(unit) {
            // If there is a path to the tile, check its movement score
            if let Some((path, _)) = self.pathfind(unit, x, y) {
                walk.update(Walk::new(path, self.search_score(x, y, unit)));
            }
        }

        AIMove::new_walk(walk.path)
    }

    // Calculate the damage score for a tile
    fn damage_score(&self, x: usize, y: usize, moves_used: u16, unit: &Unit, target: &Unit) -> f32 {
        let moves = unit.moves - moves_used;

        // Return if the cost is too high or if line of fire is blocked
        if moves < unit.weapon.tag.cost()
            || self
                .map()
                .tiles
                .line_of_fire(x, y, target.x, target.y)
                .is_some()
        {
            return 0.0;
        }

        // Calculate the chance to hit
        let chance_to_hit = chance_to_hit(x, y, target.x, target.y);

        // Return chance to hit * times the weapon can be fired * weapon damage
        chance_to_hit
            * f32::from(unit.weapon.times_can_fire(moves))
            * f32::from(unit.weapon.tag.damage())
    }

    // Calculate the search score for a tile.
    // Visible tiles that were invisible count for 1.0, while visible tiles that were foggy count for 0.1
    fn search_score(&self, x: usize, y: usize, unit: &Unit) -> f32 {
        let mut score = 0.0;

        // Loop though the tiles
        for (tile_x, tile_y) in self.map().tiles.iter() {
            // If the tile would be visible, add the score
            if self
                .map()
                .tiles
                .line_of_sight(x, y, tile_x, tile_y, unit.tag.sight(), unit.facing)
                .is_some()
            {
                score += match self.client.visibility_at(tile_x, tile_y) {
                    Visibility::Invisible => 1.0,
                    Visibility::Foggy => 0.1,
                    Visibility::Visible(_) => 0.0,
                };
            }
        }

        score
    }
}

#[test]
// This will quit after the first action
fn test_empty_ai_match() {
    use super::networking::*;
    use super::units::*;
    use settings::*;

    let settings = Settings::default();
    let mut map = Map::new(20, 20, 0.0);
    map.units
        .add(UnitType::Squaddie, Side::PlayerA, 0, 0, UnitFacing::Bottom);
    map.units
        .add(UnitType::Squaddie, Side::PlayerA, 1, 0, UnitFacing::Bottom);
    map.units
        .add(UnitType::Squaddie, Side::PlayerA, 2, 0, UnitFacing::Bottom);

    let (mut server, ai_1, ai_2) = ai_vs_ai(map, settings).unwrap();

    server.run().unwrap();
    ai_1.join().unwrap().unwrap();
    ai_2.join().unwrap().unwrap();
}
