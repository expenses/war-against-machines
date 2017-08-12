// The commands that can be issued to units

use rand;

use super::map::Map;
use super::units::{Unit, UnitSide};
use super::paths::PathPoint;
use super::animations::{Walk, Bullet, Animation, Animations};
use ui::TextDisplay;

// Whether to keep the command in the queue and an optional followup command
type CommandStatus = (bool, Option<Box<Command>>);

pub trait Command {
    // Process a command
    fn process(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay) -> CommandStatus;
}

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
}

impl Command for FinishedCommand {
     // Process the command, setting the units moves to 0 if it exists
    fn process(&mut self, map: &mut Map, _: &mut Animations, _: &mut TextDisplay) -> CommandStatus {
        if let Some(unit) = map.units.get_mut(self.unit_id) {
            unit.moves = 0;
        }

        (false, None)
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
}

impl Command for FireCommand {
    // Process the fire command, checking if the firing unit has the moves to fire,
    // if it hits, and adding the bullet to Animations
    fn process(&mut self, map: &mut Map, animations: &mut Animations, _: &mut TextDisplay) -> CommandStatus {
        // Get the target position
        let (target_x, target_y) = match map.units.get(self.target_id) {
            Some(target) => (target.x, target.y),
            _ => return (false, None)
        };

        // Fire the unit's weapon and get if the bullet will hit and the damage it will do
        let (will_hit, damage) = match map.units.get_mut(self.unit_id) {
            Some(unit) => if unit.fire_weapon() {
                (
                    unit.chance_to_hit(target_x, target_y) > rand::random::<f32>(),
                    unit.weapon.tag.damage()
                )
            } else {
                return (false, None);
            },   
            _ => return (false, None)
        };

        // Push a bullet to the animation queue
        if let Some(unit) = map.units.get(self.unit_id) {
            if let Some(target) = map.units.get(self.target_id) {
                animations.push(Animation::Bullet(Bullet::new(unit, target, will_hit)));
            }
        }

        // If the bullet will hit, return a followup damage command
        if will_hit {
            (false, Some(Box::new(DamageCommand::new(self.target_id, damage))))
        } else {
            (false, None)
        }
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
    pub fn new(unit: &Unit, map: &Map, path: Vec<PathPoint>) -> WalkCommand {
        WalkCommand {
            path,
            unit_id: unit.id,
            // Calculate the number of visible enemy units
            visible_enemies: map.visible(unit.side.enemies())
        }
    }
}

impl Command for WalkCommand {
    // Process the walk command, moving the unit one tile along the path and checking
    // if it spots an enemy unit
    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations, log: &mut TextDisplay) -> CommandStatus {
        if let Some(point) = self.path.first() {
            let moves = match map.units.get(self.unit_id) {
                Some(unit) => {
                    // If there are more visible enemies than there were when the walk started, end it
                    if map.visible(unit.side.enemies()) > self.visible_enemies {
                        // Log a message to the player that an enemy was spotted
                        if unit.side == UnitSide::Player {
                            log.append("Enemy spotted!");
                        }

                        return (false, None);
                    }

                    unit.moves
                },
                _ => return (false, None)
            };

            // If the move costs too much or if the tile is taken, end the walk
            if moves < point.cost || map.taken(point.x, point.y) {
                return (false, None);
            } else {
                // Move the unit
                if let Some(unit) = map.units.get_mut(self.unit_id) {
                    unit.move_to(point.x, point.y, point.cost);
                    map.tiles.at_mut(point.x, point.y).walk_on();
                }

                // Update the visibility of the tiles
                map.tiles.update_visibility(&map.units);

                // Add a walk to the animation queue (so that there is a delay and a footstep sound)
                animation_queue.push(Animation::Walk(Walk::new()));
            }
        }

        // Remove the point from the path
        self.path.remove(0);
        // Return whether there are still path points to process
        (!self.path.is_empty(), None)
    }
}

// Get a unit to use an item
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
}

impl Command for UseItemCommand {
    fn process(&mut self, map: &mut Map, _: &mut Animations, _: &mut TextDisplay) -> CommandStatus {
        if let Some(unit) = map.units.get_mut(self.id) {
            unit.use_item(self.item);
        }

        (false, None)
    }
}

// Damage a unit
struct DamageCommand {
    id: u8,
    damage: i16
}

impl DamageCommand {
    fn new(id: u8, damage: i16) -> DamageCommand {
        DamageCommand {
            id, damage
        }
    }
}

impl Command for DamageCommand {
    fn process(&mut self, map: &mut Map, _: &mut Animations, _: &mut TextDisplay) -> CommandStatus {
        // Deal damage to the unit and get whether it is lethal
        let lethal = match map.units.get_mut(self.id) {
            Some(target) => {
                target.health -= self.damage;
                target.health <= 0
            },
            _ => return (false, None)
        };

        // If the damage is lethal, kill the unit
        if lethal {
            map.units.kill(&mut map.tiles, self.id);
        }

        (false, None)
    }
}

// The queue of commands
pub type CommandQueue = Vec<Box<Command>>;

pub trait UpdateCommands {
    // Update the first command
    fn update(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay);
}

impl UpdateCommands for CommandQueue {
    // Update the first item of the command queue
    fn update(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay) {
        // Get the keep and optional followup command
        let (keep, followup) = match self.first_mut() {
            Some(ref mut command) => command.process(map, animations, log),
            _ => (true, None)
        };

        // If there was a followup command, insert it
        if let Some(command) = followup {
            self.insert(1, command);
        }

        // Remove the command if it's not wanted
        if !keep {
            self.remove(0);
        }
    }
}