use rand;

use battle::map::Map;
use battle::units::UnitSide;
use battle::paths::PathPoint;
use battle::animations::{Walk, Bullet, Dying, Animation, Animations};
use utils::chance_to_hit;

pub struct FinishedCommand {
    unit_id: usize
}

impl FinishedCommand {
    pub fn new(unit_id: usize) -> FinishedCommand {
        FinishedCommand {
            unit_id
        }
    }

    fn process(&self, map: &mut Map) {
        if let Some(unit) = map.units.get_mut(self.unit_id) {
            unit.moves = 0;
        }
    }
}


pub struct FireCommand {
    unit_id: usize,
    target_id: usize
}

impl FireCommand {
    pub fn new(unit_id: usize, target_id: usize) -> FireCommand {
        FireCommand {
            unit_id, target_id
        }
    }

    fn process(&self, map: &mut Map, animations: &mut Animations) {
        let (target_x, target_y) = match map.units.get(self.target_id) {
            Some(target) => (target.x, target.y),
            _ => return
        };

        let (will_hit, damage) = match map.units.get_mut(self.unit_id) {
            Some(unit) => {
                if unit.moves < unit.weapon.cost {
                    return;
                }

                unit.moves -= unit.weapon.cost;

                let hit_chance = chance_to_hit(unit.x, unit.y, target_x, target_y);
                let random = rand::random::<f32>();

                (hit_chance > random, unit.weapon.damage)
            }
            _ => return
        };

        // Add a bullet to the array for drawing
        animations.push(Animation::Bullet(Bullet::new(self.unit_id, self.target_id, will_hit, &map.units)));

        if let Some(target) = map.units.get_mut(self.target_id) {
            if will_hit {
                target.health -= damage;
            }

            if target.health <= 0 {
                animations.push(Animation::Dying(Dying::new(self.target_id)));
            }
        }
    }
}

pub struct WalkCommand {
    unit_id: usize,
    visible_units: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    pub fn new(unit_id: usize, map: &Map, path: Vec<PathPoint>) -> WalkCommand {
        let visible_units = match map.units.get(unit_id).unwrap().side {
            UnitSide::Friendly => map.visible(UnitSide::Enemy),
            UnitSide::Enemy => map.visible(UnitSide::Friendly)
        };

        WalkCommand {
            unit_id, path, visible_units
        }
    }

    fn process(&mut self, map: &mut Map, animation_queue: &mut Animations) -> bool {
        let (x, y, cost) = {
            let point = &self.path[0];
            (point.x, point.y, point.cost)
        };

        if let Some(unit) = map.units.get(self.unit_id) {
            if match unit.side {
                UnitSide::Friendly => map.visible(UnitSide::Enemy),
                UnitSide::Enemy => map.visible(UnitSide::Friendly)
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
        
        self.path.len() == 0
    }
}

pub enum Command {
    Fire(FireCommand),
    Walk(WalkCommand),
    Finished(FinishedCommand)
}


pub struct CommandQueue {
    commands: Vec<Command>
}

impl CommandQueue {
    pub fn new() -> CommandQueue {
        CommandQueue {
            commands: Vec::new()
        }
    }

    pub fn update(&mut self, map: &mut Map, animations: &mut Animations) {
        let finished = match self.commands.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => {fire.process(map, animations); true},
            Some(&mut Command::Walk(ref mut walk)) => walk.process(map, animations),
            Some(&mut Command::Finished(ref mut finished)) => {finished.process(map); true}
            _ => false
        };

        if finished {
            self.commands.remove(0);
        }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}