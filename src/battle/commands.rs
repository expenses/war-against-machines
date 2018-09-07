use super::map::*;
use super::units::*;
use super::paths::*;
use super::responses::*;

use utils::*;
use resources::*;
use rand::*;

use std::collections::*;

pub struct ServerResponses {
    player_a: Vec<Response>,
    player_b: Vec<Response>
}

impl ServerResponses {
    pub fn new() -> Self {
        Self {
            player_a: Vec::new(),
            player_b: Vec::new()
        }
    }

    fn push_if_predicate<P: Fn(Side) -> bool>(&mut self, response: Response, predicate: P) {
        if predicate(Side::PlayerA) {
            self.push(Side::PlayerA, response.clone());
        }

        if predicate(Side::PlayerB) {
            self.push(Side::PlayerB, response);
        }
    }

    pub fn push(&mut self, side: Side, response: Response) {
        match side {
            Side::PlayerA => self.player_a.push(response),
            Side::PlayerB => self.player_b.push(response)
        }
    }

    pub fn push_both(&mut self, response: Response) {
        self.push(Side::PlayerA, response.clone());
        self.push(Side::PlayerB, response);
    }

    pub fn push_and_update_state(&mut self, map: &mut Map) {
        self.push(Side::PlayerA, Response::new_state(map, Side::PlayerA));
        self.push(Side::PlayerB, Response::new_state(map, Side::PlayerB));
    }

    pub fn push_message(&mut self, message: String) {
        self.push_both(Response::Message(message));
    }

    pub fn split(self) -> (Vec<Response>, Vec<Response>) {
        (self.player_a, self.player_b)
    }
}

pub fn turn_command(map: &mut Map, id: u8, new_facing: UnitFacing, responses: &mut ServerResponses) {
    let current_facing = map.units.get(id).unwrap().facing;

    // todo: animate turning

    let (cost, _) = current_facing.rotation_cost_and_direction(new_facing);

    {
        let unit = map.units.get_mut(id).unwrap();
        if unit.moves >= cost {
            unit.facing = new_facing;
            unit.moves -= cost;
        }
    }

    responses.push_and_update_state(map);
}

pub fn move_command(map: &mut Map, id: u8, path: Vec<UnitFacing>, responses: &mut ServerResponses) {
    let side = map.units.get(id).unwrap().side;
    let visible_enemies = VisibleEnemies::new(side, map);

    for facing in path {
        let (moves, current_point) = {
            let unit = map.units.get(id).unwrap();

            if visible_enemies.new_enemy(unit.side, map).is_some() {
                return;
            }

            (unit.moves, PathPoint::from(unit))
        };

        let future_point = current_point.neighbours(map).into_iter()
            .map(|(point, _)| point)
            .find(|point| point.facing == facing);

        let future_point = match future_point {
            Some(point) => point,
            None => return
        };

        if moves < future_point.cost {
            return;
        }

        // Move the unit
        {
            map.units.get_mut(id).unwrap().move_to(&future_point);
            map.tiles.at_mut(future_point.x, future_point.y).walk_on();
        }

        responses.push_and_update_state(map);
        responses.push_both(Response::SoundEffect(SoundEffect::Walk));
        responses.push_both(Response::Walk(0.0));
    }
}

pub fn use_item_command(map: &mut Map, id: u8, item: usize, responses: &mut ServerResponses) {
    map.units.get_mut(id).unwrap().use_item(item);
    responses.push_and_update_state(map);
}

pub fn pickup_item_command(map: &mut Map, id: u8, item: usize, responses: &mut ServerResponses) {
    map.units.get_mut(id).unwrap().pickup_item(&mut map.tiles, item);
    responses.push_and_update_state(map);
}

pub fn drop_item_command(map: &mut Map, id: u8, item: usize, responses: &mut ServerResponses) {
    map.units.get_mut(id).unwrap().drop_item(&mut map.tiles, item);
    responses.push_and_update_state(map);
}

pub fn throw_item_command(map: &mut Map, id: u8, item: usize, x: usize, y: usize, responses: &mut ServerResponses) {
    let item = {
        let (item, unit_x, unit_y) = {
            let unit = map.units.get_mut(id).unwrap();
            // todo: item cost validation should happen in unit.inventory_remove/inventory_add functions so its ensured by the api
            if let Some(item) = unit.inventory_remove(item) {
                if !distance_under(x, y, unit.x, unit.y, unit.tag.throw_distance()) || ITEM_COST > unit.moves {
                    return;
                }

                unit.moves -= ITEM_COST;

                (item, unit.x, unit.y)
            } else {
                return;
            }
        };

        responses.push_if_predicate(
            Response::new_thrown_item(item.image(), unit_x, unit_y, x, y),
            |side| {
                map.tiles.visibility_at(unit_x, unit_y, side).is_visible() ||
                map.tiles.visibility_at(x, y, side).is_visible()
            }
        );

        item
    };

    if let Some((damage, radius)) = item.as_explosive() {
        explosion(map, x, y, damage, radius, responses);
    } else {
        map.tiles.drop(x, y, item);
    }

    responses.push_and_update_state(map);
}

pub fn fire_command(map: &mut Map, id: u8, mut target_x: usize, mut target_y: usize, responses: &mut ServerResponses) {
    // Fire the unit's weapon and get if the bullet will hit and the damage it will do
    let (will_hit, damage, unit_x, unit_y) = {
        let unit = map.units.get_mut(id).unwrap();
        
        if unit.fire_weapon() {
            (unit.chance_to_hit(target_x, target_y) > random::<f32>(), unit.weapon.tag.damage(), unit.x, unit.y)
        } else {
            return;
        }
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

    // Push a bullet to the sides that can see it

    {
        let unit = map.units.get(id).unwrap();
        responses.push_both(Response::SoundEffect(unit.weapon.tag.fire_sound()));

        responses.push_if_predicate(
            Response::new_bullet(unit, target_x, target_y, will_hit, map),
            |side| {
                map.tiles.visibility_at(unit.x, unit.y, side).is_visible() ||
                map.tiles.visibility_at(target_x, target_y, side).is_visible()
            }
        );
    }

    responses.push_and_update_state(map);
}

fn explosion(map: &mut Map, x: usize, y: usize, damage: i16, radius: f32, responses: &mut ServerResponses) {
    let affected_tiles: HashSet<_> = map.tiles.iter()
        .filter(|&(tile_x, tile_y)| distance_under(x, y, tile_x, tile_y, radius)).collect();

    for (i, &(tile_x, tile_y)) in affected_tiles.iter().enumerate() {
        let is_last = i == affected_tiles.len() - 1;

        // push explosion to sides that can see it
        responses.push_if_predicate(
            Response::new_explosion(tile_x, tile_y, x, y, is_last),
            |side| map.tiles.visibility_at(tile_x, tile_y, side).is_visible()
        );     
    }

    for &(x, y) in &affected_tiles {
        damage_tile(map, x, y, damage);

        if !map.tiles.horizontal_clear(x, y) && (x == 0 || affected_tiles.contains(&(x - 1, y))) {
            damage_wall(map, x, y, damage, WallSide::Left);
        }

        if !map.tiles.vertical_clear(x, y) && (y == 0 || affected_tiles.contains(&(x, y - 1))) {
            damage_wall(map, x, y, damage, WallSide::Top);
        }
    }
}


fn damage_tile(map: &mut Map, x: usize, y: usize, damage: i16) {
    // Deal damage to the unit and get whether it is lethal
    if let Some((id, lethal)) = map.units.at_mut(x, y).map(|unit| (unit.id, unit.damage(damage))) {
        // If the damage is lethal, kill the unit
        if lethal {
            map.units.kill(&mut map.tiles, id);
        }
    } else {
        // Decorate the area with a crater
        map.tiles.at_mut(x, y).decoration = Some(Image::Crater);
    }
}

fn damage_wall(map: &mut Map, x: usize, y: usize, damage: i16, side: WallSide) {
    let walls = &mut map.tiles.at_mut(x, y).walls;

    let destroyed = match side {
        WallSide::Left => walls.left.as_mut(),
        WallSide::Top => walls.top.as_mut()
    }.map(|wall| wall.damage(damage))
    .unwrap_or(false);

    if destroyed {
        match side {
            WallSide::Left => walls.left = None,
            WallSide::Top => walls.top = None
        }
    }
}
