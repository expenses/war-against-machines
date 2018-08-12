// float_cmp is an annoying clippy lint
#![cfg_attr(feature = "cargo-clippy", allow(float_cmp))]

use super::map::*;
use super::units::*;
use rand;
use rand::distributions::*;

use weapons::*;

use ui::*;
use context::*;
use utils::*;
use resources::*;
use std::f32::consts::PI;

pub fn process_animations(animations: &mut Vec<Animation>, dt: f32, map: &mut Map, ctx: &mut Context, log: &mut TextDisplay) {
    let mut i = 0;

    while i < animations.len() {
        let status = animations[i].step(dt, map, ctx, log);

        if status.finished {
            animations.remove(0);
        } else {
            i += 1;
        }

        if status.blocking {
            break;
        }
    }
}

struct Status {
    finished: bool,
    blocking: bool
}

#[derive(Debug, Clone)]
pub enum Animation {
    Walk(f32),
    NewState(Box<Map>),
    EnemySpotted {
        x: usize,
        y: usize
    },
    ThrownItem(ThrownItem),
    Explosion(Explosion),
    Bullet(Bullet)
}

impl Animation {
    pub fn new_state(map: &mut Map, side: Side) -> Self {
        Animation::NewState(Box::new(map.clone_visible(side)))
    }

    pub fn new_explosion(x: usize, y: usize, center_x: usize, center_y: usize, blocking: bool) -> Self {
        Animation::Explosion(Explosion::new(x, y, center_x, center_y, blocking))
    }

    pub fn new_thrown_item(image: Image, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {
        Animation::ThrownItem(ThrownItem::new(image, start_x, start_y, end_x, end_y))
    }

    pub fn new_bullet(unit: &Unit, target_x: usize, target_y: usize, will_hit: bool, map: &Map) -> Self {
        Animation::Bullet(Bullet::new(unit, target_x, target_y, will_hit, map))
    }

    fn step(&mut self, dt: f32, map: &mut Map, ctx: &mut Context, log: &mut TextDisplay) -> Status {
        match *self {
            Animation::NewState(ref new_map) => {
                map.clone_from(&new_map);
                Status {finished: true, blocking: false}
            },
            Animation::EnemySpotted {x, y} => {
                log.append(&format!("Enemy spotted at ({}, {})!", x, y));
                // todo: set camera location to enemy location perhaps?
                Status {finished: true, blocking: false}
            },
            Animation::Walk(ref mut time) => {
                if *time == 0.0 {
                    ctx.play_sound(&SoundEffect::Walk);
                }

                *time += dt;

                Status {finished: *time > 0.2, blocking: true}
            },
            Animation::Explosion(ref mut explosion) => explosion.step(dt),
            Animation::ThrownItem(ref mut item) => item.step(dt),
            Animation::Bullet(ref mut bullet) => bullet.step(ctx, dt)
        }
    }

    pub fn as_explosion(&self) -> Option<&Explosion> {
        match *self {
            Animation::Explosion(ref explosion) if explosion.time > 0.0 => Some(explosion),
            _ => None
        }
    }

    pub fn as_thrown_item(&self) -> Option<&ThrownItem> {
        match *self {
            Animation::ThrownItem(ref item) => Some(item),
            _ => None
        }
    }

    pub fn as_bullet(&self) -> Option<&Bullet> {
        match *self {
            Animation::Bullet(ref bullet) if !bullet.finished => Some(bullet),
            _ => None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Explosion {
    x: usize,
    y: usize,
    time: f32,
    blocking: bool
}

impl Explosion {
    const DURATION: f32 = 0.25;
    const SPEED: f32 = 10.0;

    fn new(x: usize, y: usize, center_x: usize, center_y: usize, blocking: bool) -> Self {
        Self {
            x, y, blocking,
            time: -distance(x, y, center_x, center_y) / Self::SPEED
        }
    }

    // Which explosion image to use
    pub fn image(&self) -> Image {
        let time_percentage = self.time / Self::DURATION;

        if time_percentage < (1.0 / 3.0) {
            Image::Explosion1
        } else if time_percentage < (2.0 / 3.0) {
            Image::Explosion2
        } else {
            Image::Explosion3
        }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    fn step(&mut self, dt: f32) -> Status {
        self.time += dt;        
        
        Status {finished: self.time > Self::DURATION, blocking: self.blocking}
    }
}

#[derive(Debug, Clone)]
pub struct ThrownItem {
    image: Image,
    start_x: f32,
    start_y: f32,
    progress: f32,
    increment: f32,
    end_x: f32,
    end_y: f32
}

impl ThrownItem {
    // Items can be thrown 7.5 tiles a second
    const TIME: f32 = 7.5;
    // And reach a peak height of 1 tile
    const ITEM_HEIGHT: f32 = 1.0;
    // Min time of the equiv of 5 tiles
    //const MIN: f32 = 5.0;

    fn new(image: Image, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Self {        
        Self {
            image,
            start_x: start_x as f32,
            start_y: start_y as f32,
            end_x: end_x as f32,
            end_y: end_y as f32,
            increment: Self::TIME / distance(start_x, start_y, end_x, end_y).min(Self::TIME),
            progress: 0.0,
        }
    }

    pub fn image(&self) -> Image {
        self.image
    }

    pub fn height(&self) -> f32 {
        (self.progress * PI).sin() * Self::ITEM_HEIGHT
    }

    // Interpolate the x position
    pub fn x(&self) -> f32 {
        lerp(self.start_x, self.end_x, self.progress)
    }

    // Interpolate the y position
    pub fn y(&self) -> f32 {
        lerp(self.start_y, self.end_y, self.progress)
    }

    // Update the progress and the height
    fn step(&mut self, dt: f32) -> Status {
        self.progress += self.increment * dt;
        
        Status {finished: self.progress > 1.0, blocking: true}
    }
}

const MARGIN: f32 = 5.0;

// Extrapolate two points on the map to get the point at which a bullet 
// would go off the map
fn extrapolate(x_1: f32, y_1: f32, x_2: f32, y_2: f32, map: &Map) -> (f32, f32) {
    // Get the min and max edges
    let min_x = -MARGIN;
    let min_y = -MARGIN;
    let max_x = map.tiles.width() as f32 + MARGIN;
    let max_y = map.tiles.height() as f32 + MARGIN;
    
    // get the relevant edges
    let relevant_x = if x_2 > x_1 {max_x} else {min_x};
    let relevant_y = if y_2 > y_1 {max_y} else {min_y};

    // If the line is straight just change the x or y coord
    if x_2 == x_1 {
        (x_1, relevant_y)
    } else if y_2 == y_1 {
        (relevant_x, y_1)
    } else {(
        // Extrapolate the values by the difference to an edge and clamp
        clamp(x_2 + ((x_2 - x_1) / (y_2 - y_1)) * (relevant_y - y_2), min_x, max_x),
        clamp(y_2 + ((y_2 - y_1) / (x_2 - x_1)) * (relevant_x - x_2), min_y, max_y)
    )}
}

// A bullet animation for drawing on the screen
#[derive(Debug, Clone)]
pub struct Bullet {
    x: f32,
    y: f32,
    direction: f32,
    time: f32,
    finished: bool,
    weapon_type: WeaponType,
    target_x: f32,
    target_y: f32,
}

impl Bullet {
    // Bullets travel 30 tiles a second
    const SPEED: f32 = 30.0;
    // The minimum length of time for a bullet animation is a quarter of a second
    const MIN_TIME: f32 = 0.25;

    // Create a new bullet based of the firing unit and the target unit
    fn new(unit: &Unit, target_x: usize, target_y: usize, will_hit: bool, map: &Map) -> Self {
        let x = unit.x as f32;
        let y = unit.y as f32;
        let mut target_x = target_x as f32;
        let mut target_y = target_y as f32;

        // Calculate the direction of the bullet
        let mut direction = direction(x, y, target_x, target_y);

        // If the bullet won't hit the target, change the direction slightly
        if !will_hit {
            direction += Range::new(-0.2, 0.2).sample(&mut rand::thread_rng());
            
            let (x, y) = extrapolate(x, y, x + direction.cos(), y + direction.sin(), map);

            target_x = x;
            target_y = y;
        }

        Self {
           x, y, direction, target_x, target_y,
           // Get the type of the firing weapon
           weapon_type: unit.weapon.tag,
           // The bullet hasn't started moving
           time: 0.0,
           finished: false
        }
    }
    
    // Get the image of the bullet
    pub fn image(&self) -> Image {
        self.weapon_type.bullet()
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    // Move the bullet a step and work out if its still going or not
    fn step(&mut self, ctx: &mut Context, dt: f32) -> Status {
        // If the bullet hasn't started moving, play its sound effect
        if self.time == 0.0 {
            ctx.play_sound(&self.weapon_type.fire_sound());
        }

        // Increment the time
        self.time += dt;

        // If the bullet is still moving
        if !self.finished {
            // Work out if the bullet started to the left/right and above/below the target
            let left = self.x < self.target_x;
            let above = self.y < self.target_y;

            // Move the bullet
            self.x += self.direction.cos() * Self::SPEED * dt;
            self.y += self.direction.sin() * Self::SPEED * dt;
            
            // Set if the bullet is past the target
            self.finished = left != (self.x < self.target_x) || above != (self.y < self.target_y);
        }

        // If the bullet hasn't finished or is under the minimum time
        Status {
            finished: self.finished && self.time > Self::MIN_TIME,
            blocking: true
        }
    }
}

// test extrapolation
#[test]
fn extrapolation_tests() {
    let map = Map::new(20, 10, 1.0);

    // Lateral directions
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 1.0, &map), (25.0, 1.0));
    assert_eq!(extrapolate(1.0, 1.0, 1.0, 2.0, &map), (1.0, 15.0));
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 1.0, &map), (-5.0, 1.0));
    assert_eq!(extrapolate(1.0, 1.0, 1.0, 0.0, &map), (1.0, -5.0));

    // Diagonal directions
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 0.0, &map), (-5.0, -5.0));
    assert_eq!(extrapolate(1.0, 1.0, 0.0, 2.0, &map), (-5.0, 7.0));
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 0.0, &map), (7.0, -5.0));
    assert_eq!(extrapolate(1.0, 1.0, 2.0, 2.0, &map), (15.0, 15.0));

    assert_eq!(extrapolate(0.0, 0.0, 2.0, 1.0, &map), (25.0, 12.5));
    assert_eq!(extrapolate(0.0, 0.0, 1.0, 2.0, &map), (7.5, 15.0));
}