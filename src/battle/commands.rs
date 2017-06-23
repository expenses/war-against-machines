extern crate rand;

use battle::map::Map;
use battle::paths::PathPoint;
use battle::animations::{Walk, Bullet, Dying, Animation, AnimationQueue};
use utils::chance_to_hit;

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

    fn process(&self, map: &mut Map, animation_queue: &mut AnimationQueue) {
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
        animation_queue.push(Animation::Bullet(Bullet::new(self.unit_id, self.target_id, will_hit, &map.units)));

        if let Some(target) = map.units.get_mut(self.target_id) {
            if will_hit {
                target.health -= damage;
            }

            if target.health <= 0 {
                animation_queue.push(Animation::Dying(Dying::new(self.target_id)));
            }
        }
    }
}

pub struct WalkCommand {
    unit_id: usize,
    path: Vec<PathPoint>,
}

impl WalkCommand {
    pub fn new(unit_id: usize, path: Vec<PathPoint>) -> WalkCommand {
        WalkCommand {
            unit_id, path
        }
    }

    fn process(&mut self, map: &mut Map, animation_queue: &mut AnimationQueue) -> bool {
        let (x, y, cost) = {
            let point = &self.path[0];
            (point.x, point.y, point.cost)
        };

        if let Some(unit) = map.units.get(self.unit_id) {
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
    Walk(WalkCommand)
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

    pub fn update(&mut self, map: &mut Map, animation_queue: &mut AnimationQueue) {
        let finished = match self.commands.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => {fire.process(map, animation_queue); true},
            Some(&mut Command::Walk(ref mut walk)) => walk.process(map, animation_queue),
            _ => false
        };

        if finished {
            self.commands.remove(0);
        }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }
}