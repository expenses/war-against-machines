// The commands that can be issued to units

// `Command::new` functions return a `Command` instead
// of `Self` for convenience, so turn the clippy lint off
#![cfg_attr(feature = "cargo-clippy", allow(new_ret_no_self))]

use rand;

use super::map::Map;
use super::units::{Unit, UnitSide};
use super::paths::PathPoint;
use super::animations::{Walk, Bullet, Animation, Animations};
use ui::TextDisplay;

// A response returned by a command
struct Response {
    keep: bool,
    wait: bool,
    follow_up: Option<Command>
}

// The default response
impl Default for Response {
    fn default() -> Response {
        Response {
            keep: false,
            wait: false,
            follow_up: None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Command {
    Finished(FinishedCommand),
    Fire(FireCommand),
    Walk(WalkCommand),
    UseItem(UseItemCommand),
    Damage(DamageCommand)
}

// Finish a units moves for a turn by setting them to 0
#[derive(Debug, PartialEq)]
pub struct FinishedCommand {
    unit_id: u8
}

impl FinishedCommand {
    // Create a new finished command
    pub fn new(unit_id: u8) -> Command {
        Command::Finished(FinishedCommand {
            unit_id
        })
    }

    // Process the command, setting the units moves to 0 if it exists
    fn process(&self, map: &mut Map) -> Response {
        if let Some(unit) = map.units.get_mut(self.unit_id) {
            unit.moves = 0;
        }

        Response::default()
    }
}

// Get one unit to fire on another
#[derive(Debug, PartialEq)]
pub struct FireCommand {
    unit_id: u8,
    x: usize,
    y: usize
}

impl FireCommand {
    // Create a new fire command
    pub fn new(unit_id: u8, x: usize, y: usize) -> Command {
        Command::Fire(FireCommand {
            unit_id, x, y
        })
    }

    // Process the fire command, checking if the firing unit has the moves to fire,
    // if it hits, and adding the bullet to Animations
    fn process(&mut self, map: &mut Map, animations: &mut Animations) -> Response {
        let mut response = Response::default();

        // Fire the unit's weapon and get if the bullet will hit and the damage it will do
        let (mut will_hit, damage) = match map.units.get_mut(self.unit_id) {
            Some(unit) => if unit.fire_weapon() {
                (unit.chance_to_hit(self.x, self.y) > rand::random::<f32>(), unit.weapon.tag.damage())
            } else {
                return response;
            },   
            _ => return response
        };

        // If the bullet will hit at enemy, return a followup damage command
        response.follow_up = if let Some(unit) = map.units.at(self.x, self.y) {
            if will_hit { Some(DamageCommand::new(unit.id, damage)) } else { None }
        } else {
            will_hit = false;
            None
        };

        // Push a bullet to the animation queue
        if let Some(unit) = map.units.get(self.unit_id) {
            animations.push(Animation::Bullet(Bullet::new(unit, self.x, self.y, will_hit, map)));
            response.wait = true;
        }

        response
    }
}

// Move a unit along a path, checking if it spots an enemy unit along the way
#[derive(Debug, PartialEq)]
pub struct WalkCommand {
    unit_id: u8,
    visible_enemies: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    // Create a new walk command
    pub fn new(unit: &Unit, map: &Map, path: Vec<PathPoint>) -> Command {
        Command::Walk(WalkCommand {
            path,
            unit_id: unit.id,
            // Calculate the number of visible enemy units
            visible_enemies: map.visible(unit.side.enemies())
        })
    }
    
    // Process the walk command, moving the unit one tile along the path and checking
    // if it spots an enemy unit
    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations, log: &mut TextDisplay) -> Response {
        let mut response = Response::default();

        if let Some(point) = self.path.first() {
            let moves = match map.units.get(self.unit_id) {
                Some(unit) => {
                    // If there are more visible enemies than there were when the walk started, end it
                    if map.visible(unit.side.enemies()) > self.visible_enemies {
                        // Log a message to the player that an enemy was spotted
                        if unit.side == UnitSide::Player {
                            log.append("Enemy spotted!");
                        }

                        return response;
                    }

                    unit.moves
                },
                _ => return response
            };

            // If the move costs too much or if the tile is taken, end the walk
            if moves < point.cost || map.taken(point.x, point.y) {
                return response;
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
                response.wait = true;
            }
        }

        // Remove the point from the path
        self.path.remove(0);
        // Return whether there are still path points to process
        response.keep = !self.path.is_empty();

        response
    }
}

// Get a unit to use an item
#[derive(Debug, PartialEq)]
pub struct UseItemCommand {
    id: u8,
    item: usize
}

impl UseItemCommand {
    pub fn new(id: u8, item: usize) -> Command {
        Command::UseItem(UseItemCommand {
            id, item
        })
    }
    
    fn process(&mut self, map: &mut Map) -> Response {
        if let Some(unit) = map.units.get_mut(self.id) {
            unit.use_item(self.item);
        }

        Response::default()
    }
}

// Damage a unit
#[derive(Debug, PartialEq)]
pub struct DamageCommand {
    id: u8,
    damage: i16
}

impl DamageCommand {
    fn new(id: u8, damage: i16) -> Command {
        Command::Damage(DamageCommand {
            id, damage
        })
    }
    
    fn process(&mut self, map: &mut Map) -> Response {
        let response = Response::default();

        // Deal damage to the unit and get whether it is lethal
        let lethal = match map.units.get_mut(self.id) {
            Some(target) => {
                target.health -= self.damage;
                target.health <= 0
            },
            _ => return response
        };

        // If the damage is lethal, kill the unit
        if lethal {
            map.units.kill(&mut map.tiles, self.id);
        }

        response
    }
}

pub struct CommandQueue {
    pub commands: Vec<Command>,
    wait_for_animations: bool
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue {
            commands: Vec::new(),
            wait_for_animations: false
        }
    }

    // Push a new command onto the queue
    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn update(&mut self, map: &mut Map, animations: &mut Animations, log: &mut TextDisplay) {
        while !self.is_empty() && (!self.wait_for_animations || animations.is_empty()) {
            // Get the command response
            if let Some(response) = self.commands.first_mut().map(|command| match *command {
                Command::Fire(ref mut command) => command.process(map, animations),
                Command::Walk(ref mut command) => command.process(map, animations, log),
                Command::Finished(ref mut command) => command.process(map),
                Command::UseItem(ref mut command) => command.process(map),
                Command::Damage(ref mut command) => command.process(map)
            }) {
                // If there was a followup command, insert it
                if let Some(command) = response.follow_up {
                    self.commands.insert(1, command);
                }

                // Remove the command if it's not wanted
                if !response.keep {
                    self.commands.remove(0);
                }

                self.wait_for_animations = response.wait;
            }
        }
    }

    // Is the queue empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}