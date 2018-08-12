use super::map::*;
use super::units::*;
use super::paths::*;
use super::animations::*;

use utils::*;
use resources::*;
use rand::*;

use std::collections::*;

pub struct ServerAnimations {
    player_a: Vec<Animation>,
    player_b: Vec<Animation>
}

impl ServerAnimations {
    pub fn new() -> Self {
        Self {
            player_a: Vec::new(),
            player_b: Vec::new()
        }
    }

    fn push(&mut self, side: Side, animation: Animation) {
        if side == Side::PLAYER_A {
            self.player_a.push(animation);
        } else {
            self.player_b.push(animation);
        }
    }

    fn push_both(&mut self, animation: Animation) {
        self.push(Side::PLAYER_A, animation.clone());
        self.push(Side::PLAYER_B, animation);
    }

    fn push_state(&mut self, map: &mut Map) {
        self.push(Side::PLAYER_A, Animation::new_state(map, Side::PLAYER_A));
        self.push(Side::PLAYER_B, Animation::new_state(map, Side::PLAYER_B));
    }

    pub fn split(self) -> (Vec<Animation>, Vec<Animation>) {
        (self.player_a, self.player_b)
    }
}


struct VisibleEnemies {
    positions: Vec<(usize, usize)>
}

impl VisibleEnemies {
    fn new(unit: &Unit, map: &Map) -> Self {
        Self {
            positions: map.visible(unit.side.enemies()).map(|unit| (unit.x, unit.y)).collect()
        }
    }

    fn new_enemy(&self, unit: &Unit, map: &Map) -> Option<(usize, usize)> {
        for enemy in map.visible(unit.side.enemies()) {
            let position = (enemy.x, enemy.y);
            if !self.positions.contains(&position) {
                return Some(position);
            }
        }

        None
    }
}

pub fn turn_command(map: &mut Map, id: u8, new_facing: UnitFacing, animations: &mut ServerAnimations) {
    let visible_enemies = VisibleEnemies::new(&map.units.get(id).unwrap(), map);

    let side = map.units.get(id).unwrap().side;

    // Todo: turning should have a cost
    map.units.get_mut(id).unwrap().facing = new_facing;

    animations.push_state(map);

    if let Some((x, y)) = visible_enemies.new_enemy(map.units.get(id).unwrap(), map) {
        animations.push(side, Animation::EnemySpotted {x, y});
    }
}

// todo: decide to do unit verification in commands or beforehand

pub fn move_command(map: &mut Map, id: u8, path: Vec<PathPoint>, animations: &mut ServerAnimations) {
    let visible_enemies = VisibleEnemies::new(map.units.get(id).unwrap(), map);

    let side = map.units.get(id).unwrap().side;

    for point in path {
        if let Some(unit) = map.units.get(id) {
            if let Some((x, y)) = visible_enemies.new_enemy(unit, map) {
                animations.push(side, Animation::EnemySpotted {x, y});
                return;
            }
        }

        let moves = map.units.get(id).unwrap().moves;

        if moves < point.cost || map.taken(point.x, point.y) {
            return;
        }

        // Move the unit
        if let Some(unit) = map.units.get_mut(id) {
            unit.move_to(&point);
            map.tiles.at_mut(point.x, point.y).walk_on();
        }

        animations.push_state(map);
        animations.push_both(Animation::Walk(0.0));
    }
}

pub fn use_item_command(map: &mut Map, id: u8, item: usize, animations: &mut ServerAnimations) {
    map.units.get_mut(id).unwrap().use_item(item);
    animations.push_state(map);
}

pub fn pickup_item_command(map: &mut Map, id: u8, item: usize, animations: &mut ServerAnimations) {
    map.units.get_mut(id).unwrap().pickup_item(&mut map.tiles, item);
    animations.push_state(map);
}

pub fn drop_item_command(map: &mut Map, id: u8, item: usize, animations: &mut ServerAnimations) {
    map.units.get_mut(id).unwrap().drop_item(&mut map.tiles, item);
    animations.push_state(map);
}

pub fn throw_item_command(map: &mut Map, id: u8, item: usize, x: usize, y: usize, animations: &mut ServerAnimations) {
    let item = {
        let unit = map.units.get_mut(id).unwrap();
        let item = unit.inventory_remove(item).unwrap();
        animations.push_both(Animation::new_thrown_item(item.image(), unit.x, unit.y, x, y));
        item
    };

    if let Some((damage, radius)) = item.as_explosive() {
        explosion(map, x, y, damage, radius, animations);
    } else {
        map.tiles.drop(x, y, item);
    }

    animations.push_state(map);
}

pub fn fire_command(map: &mut Map, id: u8, mut target_x: usize, mut target_y: usize, animations: &mut ServerAnimations) {

    // Fire the unit's weapon and get if the bullet will hit and the damage it will do
    let (will_hit, damage, unit_x, unit_y) = match map.units.get_mut(id) {
        Some(unit) => if unit.fire_weapon() {
            (unit.chance_to_hit(target_x, target_y) > random::<f32>(), unit.weapon.tag.damage(), unit.x, unit.y)
        } else {
            return;
        },   
        _ => return
    };

    if will_hit {
        // If the bullet will hit a wall, return a damage wall command
        if let Some(((x, y), side)) = map.tiles.line_of_fire(unit_x, unit_y, target_x, target_y) {
            target_x = x as usize;
            target_y = y as usize;

            damage_wall(map, target_x, target_y, damage, side);
        // If the bullet will hit at enemy, return a followup damage command
        } else {
            damage_tile(map, target_x, target_y, damage);
        }
    }

    // Push a bullet to the animation queue
    if let Some(unit) = map.units.get(id) {
        animations.push_both(Animation::new_bullet(unit, target_x, target_y, will_hit, map));
    }

    animations.push_state(map);
}

fn explosion(map: &mut Map, x: usize, y: usize, damage: i16, radius: f32, animations: &mut ServerAnimations) {
    let tiles: HashSet<_> = map.tiles.iter().filter(|&(tile_x, tile_y)| distance_under(x, y, tile_x, tile_y, radius)).collect();


    for (i, &(tile_x, tile_y)) in tiles.iter().enumerate() {
        let last = i == tiles.len() - 1;
        animations.push_both(Animation::new_explosion(tile_x, tile_y, x, y, last));
    }

    for &(x, y) in &tiles {
        damage_tile(map, x, y, damage);

        if !map.tiles.horizontal_clear(x, y) && (x == 0 || tiles.contains(&(x - 1, y))) {
            damage_wall(map, x, y, damage, WallSide::Left);
        }

        if !map.tiles.vertical_clear(x, y) && (y == 0 || tiles.contains(&(x, y - 1))) {
            damage_wall(map, x, y, damage, WallSide::Top);
        }
    }
}


fn damage_tile(map: &mut Map, x: usize, y: usize, damage: i16) {
    // Deal damage to the unit and get whether it is lethal
    let info = map.units.at_mut(x, y).map(|unit| {
        unit.health -= damage;
        (unit.id, unit.health <= 0)
    });

    if let Some((id, lethal)) = info {
        // If the damage is lethal, kill the unit
        if lethal {
            map.units.kill(&mut map.tiles, id);
        }
    } else {
        map.tiles.at_mut(x, y).decoration = Some(Image::Crater);
    }
}

fn damage_wall(map: &mut Map, x: usize, y: usize, damage: i16, side: WallSide) {
    let walls = &mut map.tiles.at_mut(x, y).walls;

    let destroyed = match side {
        WallSide::Left => walls.left.as_mut(),
        WallSide::Top => walls.top.as_mut()
    }.map(|wall| {
        wall.health -= damage;
        wall.health <= 0
    })
    .unwrap_or(false);

    if destroyed {
        match side {
            WallSide::Left => walls.left = None,
            WallSide::Top => walls.top = None
        }
    }
}
