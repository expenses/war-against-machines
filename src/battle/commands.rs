// The commands that can be issued to units

use rand;

use battle::map::Map;
use battle::units::{Unit, UnitSide};
use battle::paths::PathPoint;
use battle::animations::{Walk, Bullet, Animation, Animations};
use weapons::WeaponType;

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
        if self.status.is_none() {
            let (target_x, target_y) = match map.units.get(self.target_id) {
                Some(target) => (target.x, target.y),
                _ => return true
            };

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
        
        if let Some((chance_to_hit, damage, bullets)) = self.status {
            let will_hit = chance_to_hit > rand::random::<f32>();

            let lethal = match map.units.get_mut(self.target_id) {
                Some(target) => {
                    if will_hit {
                        target.health -= damage;
                    }

                    target.health <= 0
                },
                _ => return true
            };

            if let Some(unit) = map.units.get(self.unit_id) {
                if let Some(target) = map.units.get(self.target_id) {
                    match unit.weapon.tag {
                        WeaponType::Shotgun => {
                            for _ in 0 .. bullets {
                                self.push_bullet(animations, unit, target, will_hit, lethal);
                            }
                            return true;
                        },
                        _ => self.push_bullet(animations, unit, target, will_hit, lethal)
                    }
                }
            }

            if bullets == 1 {
                return true;
            }

            self.status = Some((chance_to_hit, damage, bullets - 1));
        }

        false
    }

    fn push_bullet(&self, animations: &mut Animations, unit: &Unit, target: &Unit, will_hit: bool, lethal: bool) {
        animations.push(Animation::Bullet(Bullet::new(self.target_id, unit, target, will_hit, lethal)));
    }
}

// Move a unit along a path, checking if it spots an enemy unit along the way
pub struct WalkCommand {
    unit_id: usize,
    visible_units: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    // Create a new walk command
    pub fn new(unit_id: usize, map: &Map, path: Vec<PathPoint>) -> WalkCommand {
        let visible_units = match map.units.get(unit_id).unwrap().side {
            UnitSide::Player => map.visible(UnitSide::AI),
            UnitSide::AI => map.visible(UnitSide::Player)
        };

        WalkCommand {
            unit_id, path, visible_units
        }
    }

    // Process the walk command, moving the unit one tile along the path and checking
    // if it spots an enemy unit
    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations) -> bool {
        let (x, y, cost) = {
            let point = &self.path[0];
            (point.x, point.y, point.cost)
        };

        if let Some(unit) = map.units.get(self.unit_id) {
            if match unit.side {
                UnitSide::Player => map.visible(UnitSide::AI),
                UnitSide::AI => map.visible(UnitSide::Player)
            } > self.visible_units {
                return true;
            }

            if unit.moves >= cost && map.units.at(x, y).is_none() {
                animation_queue.push(Animation::Walk(Walk::new(self.unit_id, x, y, cost)));
            } else {
                return true;
            }
        }            

        self.path.remove(0);
        
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
pub struct CommandQueue {
    commands: Vec<Command>
}

impl CommandQueue {
    // Create a new CommandQueue
    pub fn new() -> CommandQueue {
        CommandQueue {
            commands: Vec::new()
        }
    }

    // Update the first item of the command queue
    pub fn update(&mut self, map: &mut Map, animations: &mut Animations) {
        let finished = match self.commands.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => fire.process(map, animations),
            Some(&mut Command::Walk(ref mut walk)) => walk.process(map, animations),
            Some(&mut Command::Finished(ref mut finished)) => {finished.process(map); true}
            _ => false
        };

        if finished {
            self.commands.remove(0);
        }
    }

    // Push a new command onto the queue
    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    // Work out if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}