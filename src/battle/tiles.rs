// The tiles in the map, and a struct to contain them

use rand;
use rand::Rng;
use line_drawing::Bresenham;

use super::units::{UnitSide, Units, Unit, UnitFacing};
use super::walls::{Walls, WallType, WallSide};
use super::iter_2d::Iter2D;
use items::Item;
use utils::{min, lerp};
use resources::Image;

use std::mem::swap;

const MIN_PIT_SIZE: usize = 2;
const MAX_PIT_SIZE: usize = 5;

// A point for line-of-sight
type Point = (isize, isize);

// Sort two points on the y axis 
fn sort(a: Point, b: Point) -> (Point, Point, bool) {
    if a.1 > b.1 {
        (b, a, true)
    } else {
        (a, b, false)
    }
}

// Convert a coord in `usize`s to a point
fn to_point(x: usize, y: usize) -> Point {
    (x as isize, y as isize)
}

// Convert a point back into `usize`s
fn from_point(point: Point) -> (usize, usize) {
    (point.0 as usize, point.1 as usize)
}

// The visibility of the tile
#[derive(Copy, Clone, Serialize, Deserialize, Debug, is_enum_variant, PartialEq)]
pub enum Visibility {
    Visible(u8),
    Foggy,
    Invisible
}

impl Visibility {
    const DAY_DARKNESS_RATE: f32 = 0.025;
    const NIGHT_DARKNESS_RATE: f32 = 0.05;

    // Get the corresponding colour for a visibility
    pub fn colour(self, light: f32) -> [f32; 4] {
        // Get the rate at which tiles get darker
        let rate = lerp(Self::NIGHT_DARKNESS_RATE, Self::DAY_DARKNESS_RATE, light);

        // Use the distance if the tile is visible
        let alpha = if let Visibility::Visible(distance) = self {
            f32::from(distance) * rate
        // Or use the maximum darkness + 0.1
        } else if self.is_foggy() {
            rate * (Unit::SIGHT * f32::from(Unit::WALK_LATERAL_COST)) + 0.1
        } else {
            0.0
        };

        [0.0, 0.0, 0.0, alpha]
    }

    // Return the distance of the tile or the maximum value 
    fn distance(self) -> u8 {
        if let Visibility::Visible(value) = self {
            value
        } else {
            u8::max_value()
        }
    }
}

// Get the highest of two visibilities
fn combine_visibilities(a: Visibility, b: Visibility) -> Visibility {
    if a.is_visible() || b.is_visible() {
        // Use the mimimum of the two distances
        Visibility::Visible(min(a.distance(), b.distance()))
    } else if a.is_foggy() || b.is_foggy() {
        Visibility::Foggy
    } else {
        Visibility::Invisible
    }
}

#[derive(Serialize, Deserialize, is_enum_variant, Debug, Clone)]
pub enum Obstacle {
    Object(Image),
    Pit(Image),
    Empty
}

// A tile in the map
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub base: Image,
    pub obstacle: Obstacle,
    pub decoration: Option<Image>,
    pub walls: Walls,
    pub player_visibility: Visibility,
    pub ai_visibility: Visibility,
    pub items: Vec<Item>
}

impl Tile {
    // Create a new tile
    fn new(base: Image) -> Tile {
        Tile {
            base,
            obstacle: Obstacle::Empty,
            decoration: None,
            walls: Walls::new(),
            player_visibility: Visibility::Invisible,
            ai_visibility: Visibility::Invisible,
            items: Vec::new()
        }
    }

    // Set the tile to be one of the pit images and remove the decoration
    fn set_pit(&mut self, pit_image: Image) {
        self.obstacle = Obstacle::Pit(pit_image);
        self.decoration = None;
    }

    // return if the tile is visible to the player
    pub fn visible(&self) -> bool {
        !self.player_visibility.is_invisible()
    }

    // Actions that occur when the tile is walked on
    pub fn walk_on(&mut self) {
        // Crush the skeleton decoration
        if let Some(Image::Skeleton) = self.decoration {
            self.decoration = Some(Image::SkeletonCracked);
        }
    }

    pub fn items_remove(&mut self, index: usize) -> Option<Item> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }
}

// A 2D array of tiles
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tiles {
    tiles: Vec<Tile>,
    pub cols: usize,
    pub rows: usize
}

impl Tiles {
    // Create a new set of tiles but do not generate it
    pub fn new(cols: usize, rows: usize) -> Tiles {
        let mut rng = rand::thread_rng();
        let mut tiles = Vec::new();

        let bases = &[Image::Base1, Image::Base2];

        for _ in 0 .. cols * rows {
            tiles.push(Tile::new(*rng.choose(bases).unwrap()));
        }

        Tiles {
            cols, rows, tiles
        }
    }

    // Generate the tiles
    pub fn generate(&mut self, units: &Units) {
        let mut rng = rand::thread_rng();
        let objects = &[Image::ObjectRebar, Image::ObjectRubble];

        for (x, y) in self.iter() {
            let tile = self.at_mut(x, y);

            let unit = units.at(x, y).is_some();

            // Add in decorations
            if rng.gen::<f32>() < 0.05 {
                tile.decoration = Some(if rng.gen::<bool>() {
                    if unit { Image::SkeletonCracked } else { Image::Skeleton }
                } else {
                    Image::Rubble
                });
            }

            // Add in objects
            if !unit && rng.gen::<f32>() < 0.05 {
                tile.obstacle = Obstacle::Object(*rng.choose(objects).unwrap());
            }
        }

        // Generate a randomly sized pit
        self.add_pit(
            rng.gen_range(MIN_PIT_SIZE, MAX_PIT_SIZE + 1),
            rng.gen_range(MIN_PIT_SIZE, MAX_PIT_SIZE + 1)
        );

        // Add in the walls
        for (x, y) in self.iter() {
            if rng.gen::<f32>() < 0.1 {
                if rng.gen::<bool>() {
                    self.add_left_wall(x, y, WallType::Ruin1);
                    self.add_top_wall(x, y + 1, WallType::Ruin1);
                } else {
                    self.add_left_wall(x + 1, y, WallType::Ruin2);
                    self.add_top_wall(x, y + 1, WallType::Ruin2);
                }
            }
        }

        // Update visibility
        self.update_visibility(units);
    }

    // Add a left wall if possible
    pub fn add_left_wall(&mut self, x: usize, y: usize, tag: WallType) {
        if x < self.cols && y < self.rows && (self.not_pit(x, y) || self.not_pit(x - 1, y)) {
            self.at_mut(x, y).walls.set_left(tag);
        }
    }

    // Add a top wall if possible
    pub fn add_top_wall(&mut self, x: usize, y: usize, tag: WallType) {
        if x < self.cols && y < self.rows && (self.not_pit(x, y) || self.not_pit(x, y - 1)) {
            self.at_mut(x, y).walls.set_top(tag);
        }
    }

    // Check if a position is in-bounds and not a pit
    fn not_pit(&self, x: usize, y: usize) -> bool {
        x < self.cols && y < self.rows && !self.at(x, y).obstacle.is_pit()
    }

    fn add_pit(&mut self, width: usize, height: usize) {
        // Generate pit position and size
        let mut rng = rand::thread_rng();

        let max_x = self.cols - width  - 1;
        let max_y = self.rows - height - 1;

        let pit_x = if max_x > 1 { rng.gen_range(1, max_x) } else { 1 };
        let pit_y = if max_y > 1 { rng.gen_range(1, max_y) } else { 1 };

        // Add pit corners
        self.at_mut(pit_x,             pit_y             ).set_pit(Image::PitTop);
        self.at_mut(pit_x,             pit_y + height - 1).set_pit(Image::PitLeft);
        self.at_mut(pit_x + width - 1, pit_y             ).set_pit(Image::PitRight);
        self.at_mut(pit_x + width - 1, pit_y + height - 1).set_pit(Image::PitBottom);

        // Add in the top/bottom pit edges and center
        for x in pit_x + 1 .. pit_x + width - 1 {
            self.at_mut(x, pit_y             ).set_pit(Image::PitTR);
            self.at_mut(x, pit_y + height - 1).set_pit(Image::PitBL);

            for y in pit_y + 1 .. pit_y + height - 1 {
                self.at_mut(x, y).set_pit(Image::PitCenter);
            }
        }

        // Add in the left/right pit edges
        for y in pit_y + 1 .. pit_y + height - 1 {
            self.at_mut(pit_x,             y).set_pit(Image::PitTL);
            self.at_mut(pit_x + width - 1, y).set_pit(Image::PitBR);
        }
    }

    // Get a reference to a tile
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        assert!(x < self.cols && y < self.rows, "Tile at ({}, {}) is out of bounds", x, y);
        &self.tiles[x * self.rows + y]
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        assert!(x < self.cols && y < self.rows, "Tile at ({}, {}) is out of bounds", x, y);
        &mut self.tiles[x * self.rows + y]
    }

    // Update the visibility of the map
    pub fn update_visibility(&mut self, units: &Units) {
        for (x, y) in self.iter() {
            let player_visible = self.tile_visible(units, &UnitSide::Player, x, y);
            let ai_visible = self.tile_visible(units, &UnitSide::AI, x, y);
            let tile = self.at_mut(x, y);
            
            // If the tile is visible set the visibility to visible, or if it was visible make it foggy
            
            if let Some(distance) = player_visible {
                tile.player_visibility = Visibility::Visible(distance);
            } else if tile.player_visibility.is_visible() {
                tile.player_visibility = Visibility::Foggy;
            }
            
            if let Some(distance) = ai_visible {
                tile.ai_visibility = Visibility::Visible(distance);
            } else if tile.ai_visibility.is_visible() {
                tile.ai_visibility = Visibility::Foggy;
            }
        }
    }

    // Drop an item onto the map
    pub fn drop(&mut self, x: usize, y: usize, item: Item) {
        self.at_mut(x, y).items.push(item);
    }

    // Drop a vec of items onto the map
    pub fn drop_all(&mut self, x: usize, y: usize, items: &mut Vec<Item>) {
        self.at_mut(x, y).items.append(items);
    }

    // Return whether there is a wall between two tiles
    fn wall_between(&self, a: Point, b: Point) -> bool {
        let ((a_x, a_y), (b_x, b_y)) = (from_point(a), from_point(b));

        ! match (b.0 - a.0, b.1 - a.1) {
            (0, 1) => self.vertical_clear(b_x, b_y),
            (1, 0) => self.horizontal_clear(b_x, b_y),
            (-1, 0) => self.horizontal_clear(a_x, a_y),
            (-1, 1) => self.diagonal_clear(a_x, b_y, false),
            (1, 1) => self.diagonal_clear(b_x, b_y, true),
            _ => unreachable!()
        }
    }

    // Return the first blocking obstacle between two points or none
    pub fn line_of_fire(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> Option<(Point, WallSide)> {
        // Convert the points to isize and sort
        let (start, end) = (to_point(start_x, start_y), to_point(end_x, end_y));
        let (start, end, reversed) = sort(start, end);

        // Create an iterator of tile steps
        let mut iter = Bresenham::new(start, end).steps()
            // Filter to steps with walls between
            .filter(|&(a, b)| self.wall_between(a, b))
            // Map to the containing tile and wall direction
            .map(|(a, b)| match (b.0 - a.0, b.1 - a.1) {
                (0, 1) => (b, WallSide::Top),
                (1, 0) => (b, WallSide::Left),
                (-1, 0) => (a, WallSide::Left),
                // For diagonal steps we have to randomly pick one of the two closest walls
                (-1, 1) | (1, 1) => {
                    let left_to_right = a.0 < b.0;

                    // Get the four walls segments between the tiles if left-to-right or their flipped equivalents
                    let (mut top, mut left, mut right, mut bottom) = if left_to_right {
                        ((b.0, a.1), (a.0, b.1), b, b)
                    } else {
                        (a, (a.0, b.1), b, (a.0, b.1))
                    };

                    // Swap the points around if the line is reversed
                    if reversed {
                        swap(&mut top, &mut bottom);
                        swap(&mut left, &mut right);
                    }

                    // Get whether each of these segments contain walls
                    let top_block = !self.horizontal_clear(top.0 as usize, top.1 as usize);
                    let left_block = !self.vertical_clear(left.0 as usize, left.1 as usize);
                    let right_block = !self.vertical_clear(right.0 as usize, right.1 as usize);
                    let bottom_block = !self.horizontal_clear(bottom.0 as usize, bottom.1 as usize);

                    // Get the pairs of walls to choose from
                    let (wall_a, wall_b) = if top_block && left_block {
                        ((top, WallSide::Left), (left, WallSide::Top))
                    } else if left_block && right_block {
                        ((left, WallSide::Top), (right, WallSide::Top))
                    } else if top_block && bottom_block {
                        ((top, WallSide::Left), (bottom, WallSide::Left))
                    } else {
                        ((bottom, WallSide::Left), (right, WallSide::Top))
                    };

                    // Choose a random wall
                    if rand::random::<bool>() { wall_a } else { wall_b }
                },
                _ => unreachable!()
            });

        // Return either the last or first wall found or none
        if reversed { iter.last() } else { iter.next() }
    }

    // Would a unit with a particular sight range be able to see from one tile to another
    // Return the number of tiles away a point is, or none if visibility is blocked
    pub fn line_of_sight(&self, a_x: usize, a_y: usize, b_x: usize, b_y: usize, sight: f32, facing: UnitFacing) -> Option<u8> {
        if facing.can_see(a_x, a_y, b_x, b_y, sight) {
            // Sort the points so that line-of-sight is symmetrical
            let (start, end, _) = sort(to_point(a_x, a_y), to_point(b_x, b_y));
            let mut distance = 0;

            for (a, b) in Bresenham::new(start, end).steps() {
                // Return if line of sight is blocked by a wall
                if self.wall_between(a, b) {
                    return None;
                }

                // Increase the distance
                distance += if a.0 == b.0 || a.1 == b.1 {
                    Unit::WALK_LATERAL_COST
                } else {
                    Unit::WALK_DIAGONAL_COST
                } as u8;
            }

            Some(distance)
        } else {
            None
        }
    }

    // Is a tile visible by any unit on a particular side
    fn tile_visible(&self, units: &Units, side: &UnitSide, x: usize, y: usize) -> Option<u8> {
        units.iter()
            .filter(|unit| unit.side == *side)
            .map(|unit| self.line_of_sight(unit.x, unit.y, x, y, unit.tag.sight(), unit.facing))
            // Get the minimum distance or none
            .fold(None, |sum, dist| sum.and_then(|sum| dist.map(|dist| min(sum, dist))).or(sum).or(dist))
    }

    // Is the wall space between two horizontal tiles empty
    pub fn horizontal_clear(&self, x: usize, y: usize) -> bool {
        self.at(x, y).walls.left.is_none()
    }

    // Is the wall space between two vertical tiles empty
    pub fn vertical_clear(&self, x: usize, y: usize) -> bool {
        self.at(x, y).walls.top.is_none()
    }

    // Is a diagonal clear
    pub fn diagonal_clear(&self, x: usize, y: usize, tl_to_br: bool) -> bool {
        if x.wrapping_sub(1) >= self.cols - 1 || y.wrapping_sub(1) >= self.rows - 1 {
            return false;
        }

        // Check the walls between the tiles

        let top = self.horizontal_clear(x, y - 1);
        let left = self.vertical_clear(x - 1, y);
        let right = self.vertical_clear(x, y);
        let bottom = self.horizontal_clear(x, y);

        // Check that there isn't a wall across the tiles and the right corners are open

        (top || bottom) && (left || right) && if tl_to_br {
            (top || left) && (bottom || right)
        } else {
            (top || right) && (bottom || left)
        }
    }

    // What should the visiblity of a left wall at a position be
    pub fn left_wall_visibility(&self, x: usize, y: usize) -> Visibility {
        let visibility = self.at(x, y).player_visibility;

        if x > 0 {
            combine_visibilities(visibility, self.at(x - 1, y).player_visibility)
        } else {
            visibility
        }
    }

    // What should the visibility of a top wall at a position be
    pub fn top_wall_visibility(&self, x: usize, y: usize) -> Visibility {
        let visibility = self.at(x, y).player_visibility;
        
        if y > 0 {
            combine_visibilities(visibility, self.at(x, y - 1).player_visibility)
        } else {
            visibility
        }
    }

    // Iterate through the rows and columns
    pub fn iter(&self) -> Iter2D {
        Iter2D::new(self.cols, self.rows)
    }

    pub fn visible_units<'a>(&'a self, units: &'a Units) -> impl Iterator<Item=&'a Unit> {
        units.iter().filter(move |unit| self.at(unit.x, unit.y).player_visibility.is_visible())
    }
}

#[test]
fn unit_visibility() {
    use super::units::UnitType;
    use super::paths::PathPoint;

    let mut tiles = Tiles::new(30, 30);
    let mut units = Units::new();
    units.add(UnitType::Squaddie, UnitSide::Player, 0, 0, UnitFacing::Bottom);
    tiles.update_visibility(&units);

    // A tile a unit is standing on should be visible with a distance of 0
    assert_eq!(tiles.at(0, 0).player_visibility, Visibility::Visible(0));
    // A far away tile should be invisible
    assert_eq!(tiles.at(29, 29).player_visibility, Visibility::Invisible);

    // A tile that was visible but is no longer should be foggy

    units.get_mut(0).unwrap().move_to(&PathPoint::new(29, 0, 0, UnitFacing::Top));
    tiles.update_visibility(&units);

    assert_eq!(tiles.at(0, 0).player_visibility, Visibility::Foggy);

    // If the unit is boxed into a corner, only it's tile should be visible

    tiles.add_left_wall(29, 0, WallType::Ruin1);
    tiles.add_top_wall(29, 1, WallType::Ruin2);

    tiles.update_visibility(&units);

    for (x, y) in tiles.iter() {
        let visibility = tiles.at(x, y).player_visibility;

        if x == 29 && y == 0 {
            assert_eq!(visibility, Visibility::Visible(0));
            assert!(tiles.at(x, y).visible());
        } else {
            assert!(!visibility.is_visible());
        }
    }
}

#[test]
fn line_of_fire() {
    let mut tiles = Tiles::new(5, 5);

    tiles.add_left_wall(1, 0, WallType::Ruin1);
    tiles.add_top_wall(0, 1, WallType::Ruin1);
    tiles.add_top_wall(1, 1, WallType::Ruin1);
    tiles.add_left_wall(1, 1, WallType::Ruin1);

    let top = Some(((1, 0), WallSide::Left));
    let left = Some(((0, 1), WallSide::Top));
    let right = Some(((1, 1), WallSide::Top));
    let bottom = Some(((1, 1), WallSide::Left));

    // Test lateral directions

    assert_eq!(tiles.line_of_fire(0, 0, 1, 0), top);
    assert_eq!(tiles.line_of_fire(0, 0, 0, 1), left);

    // Test diagonal directions

    let diag_1 = tiles.line_of_fire(0, 0, 1, 1);
    assert!(diag_1 == top || diag_1 == left);
    let diag_2 = tiles.line_of_fire(1, 1, 0, 0);
    assert!(diag_2 == bottom || diag_2 == right);
    let diag_3 = tiles.line_of_fire(0, 1, 1, 0);
    assert!(diag_3 == left || diag_3 == bottom);
    let diag_4 = tiles.line_of_fire(1, 0, 0, 1);
    assert!(diag_4 == right || diag_4 == top);

}

#[test]
fn pit_generation() {
    let mut tiles = Tiles::new(30, 30);
    tiles.generate(&Units::new());

    // At least one tile should have a pit on it
    assert!(tiles.tiles.iter().any(|tile| tile.obstacle.is_pit()));
}

#[test]
fn walk_on_tile() {
    let mut tiles = Tiles::new(30, 30);
   
    let tile = tiles.at_mut(0, 0);
    tile.decoration = Some(Image::Skeleton);
    tile.walk_on();
    assert_eq!(tile.decoration, Some(Image::SkeletonCracked));
}

#[test]
fn map_generation() {
    let units = Units::new();

    // Test generating maps at various sizes
    Tiles::new(10, 10).generate(&units);
    Tiles::new(30, 30).generate(&units);
    Tiles::new(10, 30).generate(&units);
    Tiles::new(30, 10).generate(&units);
}