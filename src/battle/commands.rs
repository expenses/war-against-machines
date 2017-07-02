// The commands that can be issued to units

use rand;

use battle::map::Map;
use battle::units::{Unit, UnitSide};
use battle::paths::PathPoint;
use battle::animations::{Walk, Bullet, Animation, Animations};

// Finish a units moves for a turn by setting them to 0
pub struct FinishedCommand {
    unit_id: usize
}

impl FinishedCommand {
    // Create a new finished command
    pub fn new(unit_id: usize) -> FinishedCommand {
        FinishedCommand {
            unit_id
        }
    }

    // Process the command, setting the units moves to 0 if it exists
    fn process(&self, map: &mut Map) {
        if let Some(unit) = map.units.get_mut(self.unit_id) {
            unit.moves = 0;
        }
    }
}

// Get one unit to fire on another
pub struct FireCommand {
    unit_id: usize,
    target_id: usize,
    status: Option<(f32, i16, usize)>
}

impl FireCommand {
    // Create a new fire command
    pub fn new(unit_id: usize, target_id: usize) -> FireCommand {
        FireCommand {
            unit_id, target_id,
            status: None
        }
    }

    // Process the fire command, checking if the firing unit has the moves to fire,
    // if it hits, and adding the bullet to Animations
    fn process(&mut self, map: &mut Map, animations: &mut Animations) -> bool {
        // If the status isn't set, firing hasn't started so get the weapon info
        if self.status.is_none() {
            // Get the target position
            let (target_x, target_y) = match map.units.get(self.target_id) {
                Some(target) => (target.x, target.y),
                _ => return true
            };

            // Expend the units moves and set the status
            match map.units.get_mut(self.unit_id) {
                Some(unit) => {
                    let info = unit.weapon.info();
                    let chance_to_hit = unit.chance_to_hit(target_x, target_y) * info.hit_modifier;

                    if unit.moves < info.cost {
                        return true;
                    }

                    unit.moves -= info.cost;
                    
                    self.status = Some((chance_to_hit, unit.weapon.damage, info.bullets));
                }
                _ => return true
            };
        }
        
        // Get out the status info
        if let Some((chance_to_hit, damage, bullets)) = self.status {
            // Calculate if the bullet will hit
            let will_hit = chance_to_hit > rand::random::<f32>();

            // Deal damage to the target and calculate if the bullet is lethal
            let lethal = match map.units.get_mut(self.target_id) {
                Some(target) => {
                    if will_hit {
                        target.health -= damage;
                    }

                    target.health <= 0
                },
                _ => return true
            };

            // Push a bullet to the animation queue
            if let Some(unit) = map.units.get(self.unit_id) {
                if let Some(target) = map.units.get(self.target_id) {
                    animations.push(Animation::Bullet(Bullet::new(self.target_id, unit, target, will_hit, lethal)));
                }
            }

            // If that was the last bullet, return
            // Otherwise, Lower the bullets in the status
            if bullets == 1 {
                return true;
            } else {
                self.status = Some((chance_to_hit, damage, bullets - 1));
            }

        }

        false
    }
}

// Calculate the visible enemy units for a unit
fn visible_enemies(map: &Map, unit: &Unit) -> usize {
    match unit.side {
        UnitSide::Player => map.visible(UnitSide::AI),
        UnitSide::AI => map.visible(UnitSide::Player)
    }
}

// Move a unit along a path, checking if it spots an enemy unit along the way
pub struct WalkCommand {
    unit_id: usize,
    visible_enemies: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    // Create a new walk command
    pub fn new(unit_id: usize, map: &Map, path: Vec<PathPoint>) -> WalkCommand {
        WalkCommand {
            unit_id, path,
            // Calculate the number of visible enemy units
            visible_enemies: visible_enemies(map, map.units.get(unit_id).unwrap())
        }
    }

    // Process the walk command, moving the unit one tile along the path and checking
    // if it spots an enemy unit
    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations) -> bool {
        // Get the path x, y and cost
        let (x, y, cost) = {
            let point = &self.path[0];
            (point.x, point.y, point.cost)
        };

        if let Some(unit) = map.units.get(self.unit_id) {
            // If there are more visible enemies now than when the walk began
            // or if the move costs too much
            // or if the tile is taken, end the walk
            if visible_enemies(map, unit) > self.visible_enemies ||
               unit.moves < cost ||
               map.units.at(x, y).is_some() {
                return true;
            } else {
                // Otherwise, add a walk to the animation queue
                animation_queue.push(Animation::Walk(Walk::new(self.unit_id, x, y, cost)));
            }
        }            

        // Remove the point from the path
        self.path.remove(0);
        // Return whether the path is empty
        self.path.is_empty()
    }
}

// The command enum for holding the different kinds of commands
pub enum Command {
    Fire(FireCommand),
    Walk(WalkCommand),
    Finished(FinishedCommand)
}

// The queue of commands
pub type CommandQueue = Vec<Command>;

pub trait UpdateCommands {
    fn update(&mut self, map: &mut Map, animations: &mut Animations);
}

impl UpdateCommands for CommandQueue {
    // Update the first item of the command queue
    fn update(&mut self, map: &mut Map, animations: &mut Animations) {
        let finished = match self.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => fire.process(map, animations),
            Some(&mut Command::Walk(ref mut walk)) => walk.process(map, animations),
            Some(&mut Command::Finished(ref mut finished)) => {finished.process(map); true}
            _ => false
        };

        if finished {
            self.remove(0);
        }
    }
}