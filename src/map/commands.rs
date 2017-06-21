extern crate rand;

use map::paths::PathPoint;
use map::units::Units;
use map::tiles::Tiles;
use map::animations::{Walk, Bullet, Dying, Animation, AnimationQueue};
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

    fn process(&self, units: &mut Units, animation_queue: &mut AnimationQueue) -> bool {
        let (will_hit, lethal) = {
            let (unit, target) = units.get_two_mut(self.unit_id, self.target_id);

            if unit.moves < unit.weapon.cost || !target.alive() {
                return true;
            }

            unit.moves -= unit.weapon.cost;

            let hit_chance = chance_to_hit(unit.x, unit.y, target.x, target.y);
            let random = rand::random::<f32>();
            let will_hit = hit_chance > random;

            // Lower the targets health
            if will_hit {
                target.health -= unit.weapon.damage;
            }

            (will_hit, target.health <= 0)
        };
        
        // Add a bullet to the array for drawing
        animation_queue.push(Animation::Bullet(Bullet::new(self.unit_id, self.target_id, will_hit, units)));

        if lethal {
            animation_queue.push(Animation::Dying(Dying::new(self.target_id)));
        }

        true
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

    fn process(&mut self, units: &mut Units, tiles: &mut Tiles, animation_queue: &mut AnimationQueue) -> bool {
        let moved = {
            let point = &self.path[0];

            let empty = units.at(point.x, point.y).is_none();

            let unit = units.get_mut(self.unit_id);

            let moved = unit.moves >= point.cost && empty;

            if moved {
                unit.move_to(point);

                animation_queue.push(Animation::Walk(Walk::new()));
            } else {
                return true;
            }

            moved
        };

        if moved {
            tiles.update_visibility(units);
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

    pub fn update(&mut self, units: &mut Units, tiles: &mut Tiles, animation_queue: &mut AnimationQueue) {
        let finished = match self.commands.first_mut() {
            Some(&mut Command::Fire(ref mut fire)) => fire.process(units, animation_queue),
            Some(&mut Command::Walk(ref mut walk)) => walk.process(units, tiles, animation_queue),
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