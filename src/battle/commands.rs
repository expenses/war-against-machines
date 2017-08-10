// The commands that can be issued to units

use rand;

use super::map::Map;
use super::units::{Unit, UnitSide};
use super::paths::PathPoint;
use super::animations::{Walk, Bullet, Animation, Animations};
use ui::TextDisplay;

// Finish a units moves for a turn by setting them to 0
pub struct FinishedCommand {
    unit_id: u8
}

impl FinishedCommand {
    // Create a new finished command
    pub fn new(unit_id: u8) -> FinishedCommand {
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
    unit_id: u8,
    target_id: u8,
}

impl FireCommand {
    // Create a new fire command
    pub fn new(unit_id: u8, target_id: u8) -> FireCommand {
        FireCommand {
            unit_id, target_id,
        }
    }

    // Process the fire command, checking if the firing unit has the moves to fire,
    // if it hits, and adding the bullet to Animations
    fn process(&mut self, map: &mut Map, animations: &mut Animations) {
        // Get the target position
        let (target_x, target_y) = match map.units.get(self.target_id) {
            Some(target) => (target.x, target.y),
            _ => return
        };

        // Fire the unit's weapon and get if the bullet will hit and the damage it will do
        let (will_hit, damage) = match map.units.get_mut(self.unit_id) {
            Some(unit) => {
                let will_hit = unit.chance_to_hit(target_x, target_y) > rand::random::<f32>();

                if unit.moves >= unit.weapon.tag.cost() {
                    unit.moves -= unit.weapon.tag.cost();
                    unit.weapon.fire();

                    (will_hit, unit.weapon.tag.damage())
                } else {
                    return;
                }   
            }
            _ => return
        };
        
        // Deal damage to the target and calculate if the bullet is lethal
        let lethal = match map.units.get_mut(self.target_id) {
            Some(target) => {
                if will_hit {
                    target.health -= damage;
                }

                target.health <= 0
            },
            _ => return
        };

        // Push a bullet to the animation queue
        if let Some(unit) = map.units.get(self.unit_id) {
            if let Some(target) = map.units.get(self.target_id) {
                animations.push(Animation::Bullet(Bullet::new(unit, target, will_hit, lethal)));
            }
        }
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
    unit_id: u8,
    visible_enemies: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    // Create a new walk command
    pub fn new(unit_id: u8, map: &Map, path: Vec<PathPoint>) -> WalkCommand {
        WalkCommand {
            unit_id, path,
            // Calculate the number of visible enemy units
            visible_enemies: visible_enemies(map, map.units.get(unit_id).unwrap())
        }
    }

    // Process the walk command, moving the unit one tile along the path and checking
    // if it spots an enemy unit
    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations, log: &mut TextDisplay) -> bool {
        // Get the path x, y and cost
        let (x, y, cost) = {
            let point = &self.path[0];
            (point.x, point.y, point.cost)
        };

        if let Some(unit) = map.units.get(self.unit_id) {
            // If there are more visible enemies than there were when the walk started, end it
            if visible_enemies(map, unit) > self.visible_enemies {
                // Log a message to the player that an enemy was spotted
                if unit.side == UnitSide::Player {
                    log.append("Enemy spotted!");
                }

                return true;
            }

            // If the move costs too much or if the tile is taken, end the walk
            if unit.moves < cost || map.units.at(x, y).is_some() {
                return true;
            } else {
                // Otherwise, add a walk to the animation queue
                animation_queue.push(Animation::Walk(Walk::new(self.unit_id, x, y, cost)));
            }
        } else {
            return true;
        }            

        // Remove the point from the path
        self.path.remove(0);
        // Return whether the path is empty
        self.path.is_empty()
    }
}

pub struct UseItemCommand {
    id: u8,
    item: usize
}

impl UseItemCommand {
    pub fn new(id: u8, item: usize) -> UseItemCommand {
        UseItemCommand {
            id, item
        }
    }

    fn process(&mut self, map: &mut Map) {
        if let Some(unit) = map.units.get_mut(self.id) {
            unit.use_item(self.item);
        }
    }
}

// The command enum for holding the different kinds of commands
pub enum Command {
    Fire(FireCommand),
    Walk(WalkCommand),
    Finished(FinishedCommand),
    UseItem(UseItemCommand)
}

// The queue of commands
pub type CommandQueue = Vec<Command>;

pub trait UpdateCommands {
    fn update(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay);
}

impl UpdateCommands for CommandQueue {
    // Update the first item of the command queue
    fn update(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay) {
        let finished = match self.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => {
                fire.process(map, animations);
                true
            },
            Some(&mut Command::Walk(ref mut walk)) => walk.process(map, animations, log),
            Some(&mut Command::Finished(ref mut finished)) => {
                finished.process(map);
                true
            },
            Some(&mut Command::UseItem(ref mut use_item)) => {
                use_item.process(map);
                true
            },
            _ => false
        };

        if finished {
            self.remove(0);
        }
    }
}