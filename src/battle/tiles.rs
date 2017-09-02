// The tiles in the map, and a struct to contain them

use rand;
use rand::Rng;
use line_drawing::Bresenham;

use std::ops::Add;

use super::units::{UnitSide, Units};
use super::walls::{Walls, WallType};
use items::Item;
use utils::distance_under;
use resources::Image;
use colours;

// A point for line-of-sight
type Point = (isize, isize);

// Sort two points on the y axis 
fn sort(a: Point, b: Point) -> (Point, Point) {
    if a.1 > b.1 {
        (b, a)
    } else {
        (a, b)
    }
}

fn point(x: usize, y: usize) -> Point {
    (x as isize, y as isize)
}

// The visibility of the tile
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum Visibility {
    Visible,
    Foggy,
    Invisible
}

impl Visibility {
    // Get the corresponding colour for a visibility
    pub fn colour(&self) -> [f32; 4] {
        if let Visibility::Foggy = *self {
            colours::FOGGY
        } else {
            colours::ALPHA
        }
    }
}

impl Add for Visibility {
    type Output = Visibility;

    // Get the highest of two visibilities
    fn add(self, other: Visibility) -> Visibility {
        if self == Visibility::Visible || other == Visibility::Visible {
            Visibility::Visible
        } else if self == Visibility::Foggy || other == Visibility::Foggy {
            Visibility::Foggy
        } else {
            Visibility::Invisible
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Obstacle {
    Object(Image),
    Pit(Image),
    Empty
}

impl Obstacle {
    // Is the obstacle spot empty
    pub fn is_empty(&self) -> bool {
        if let Obstacle::Empty = *self {
            true
        } else {
            false
        }
    }

    // Is the obstacle a pit
    pub fn is_pit(&self) -> bool {
        if let Obstacle::Pit(_) = *self {
            true
        } else {
            false
        }
    }
}

// A tile in the map
#[derive(Serialize, Deserialize)]
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
        self.player_visibility != Visibility::Invisible
    }

    // Actions that occur when the tile is walked on
    pub fn walk_on(&mut self) {
        // Crush the skeleton decoration
        if let Some(Image::Skeleton) = self.decoration {
            self.decoration = Some(Image::SkeletonCracked);
        }
    }
}

// A 2D array of tiles
#[derive(Serialize, Deserialize)]
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

        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
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
        }

        // Generate a randomly sized pit
        self.add_pit(rng.gen_range(2, 5), rng.gen_range(2, 5));

        // Add in the walls
        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
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
        }

        // Update visibility
        self.update_visibility(units);
    }

    // Add a left wall if possible
    pub fn add_left_wall(&mut self, x: usize, y: usize, tag: WallType) {
        if x < self.rows && y < self.rows && (self.not_pit(x, y) || self.not_pit(x - 1, y)) {
            self.at_mut(x, y).walls.set_left(tag);
        }
    }

    // Add a top wall if possible
    pub fn add_top_wall(&mut self, x: usize, y: usize, tag: WallType) {
        if x < self.rows && y < self.rows && (self.not_pit(x, y) || self.not_pit(x, y - 1)) {
            self.at_mut(x, y).walls.set_top(tag);
        }
    }

    // Check if a position is in-bounds and not a pit
    fn not_pit(&self, x: usize, y: usize) -> bool {
        x < self.rows && y < self.cols && !self.at(x, y).obstacle.is_pit()
    }

    fn add_pit(&mut self, width: usize, height: usize) {
        // Generate pit position and size
        let mut rng = rand::thread_rng();
        let pit_x = rng.gen_range(1, self.cols - width  - 1);
        let pit_y = rng.gen_range(1, self.rows - height - 1);

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
        assert!(x < self.rows && y < self.cols, "Tile at ({}, {}) is out of bounds", x, y);
        &self.tiles[x * self.rows + y]
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        assert!(x < self.rows && y < self.cols, "Tile at ({}, {}) is out of bounds", x, y);
        &mut self.tiles[x * self.rows + y]
    }

    // Update the visibility of the map
    pub fn update_visibility(&mut self, units: &Units) {
        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
                let player_visible = self.tile_visible(units, UnitSide::Player, x, y);
                let ai_visible = self.tile_visible(units, UnitSide::AI, x, y);
                let tile = self.at_mut(x, y);
                
                // If the tile is visible set the visibility to visible, or if it was visible make it foggy
                
                if player_visible {
                    tile.player_visibility = Visibility::Visible;
                } else if tile.player_visibility == Visibility::Visible {
                    tile.player_visibility = Visibility::Foggy;
                }
                
                if ai_visible {
                    tile.ai_visibility = Visibility::Visible;
                } else if tile.ai_visibility == Visibility::Visible {
                    tile.ai_visibility = Visibility::Foggy;
                }
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

    pub fn line_of_sight(&self, start: Point, end: Point) -> Option<(Point, Point)> {
        // Sort the points so that line-of-sight is symmetrical
        let (sorted_start, sorted_end) = sort(start, end);

        // Get the points for the whole line
        // Iterate over it in windows of 2
        let mut iterator = Bresenham::new(sorted_start, sorted_end).steps()            
            .filter(|&(a, b)| {
                let (a_x, a_y, b_x, b_y) = (a.0 as usize, a.1 as usize, b.0 as usize, b.1 as usize);
                
                // Filter out the sections that are clear
                ! match (b.0 - a.0, b.1 - a.1) {
                    (0, 1) => self.vertical_clear(b_x, b_y),
                    (1, 0) => self.horizontal_clear(b_x, b_y),
                    (-1, 0) => self.horizontal_clear(a_x, a_y),
                    (-1, 1) => self.diagonal_clear(a_x, b_y, false),
                    (1, 1) => self.diagonal_clear(b_x, b_y, true),
                    _ => unreachable!()
                }
            });

        // If the points weren't sorted, return the first wall (or none)
        // If they were, return the last wall (or none)
        if sorted_start == start {
            iterator.nth(0)
        } else {
            iterator.last()
        }
    }

    // Is a tile visible by any unit on a particular side
    fn tile_visible(&self, units: &Units, side: UnitSide, x: usize, y: usize) -> bool {
        units.iter()
            .filter(|unit| unit.side == side)
            .any(|unit| self.visible(unit.x, unit.y, x, y, unit.tag.sight()))
    }

    // Would a unit with a particular sight range be able to see from one tile to another
    pub fn visible(&self, a_x: usize, a_y: usize, b_x: usize, b_y: usize, sight: f32) -> bool {
        distance_under(a_x, a_y, b_x, b_y, sight) &&
        self.line_of_sight(point(a_x, a_y), point(b_x, b_y)).is_none()
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
            visibility + self.at(x - 1, y).player_visibility
        } else {
            visibility
        }
    }

    // What should the visibility of a top wall at a position be
    pub fn top_wall_visibility(&self, x: usize, y: usize) -> Visibility {
        let visibility = self.at(x, y).player_visibility;
        
        if y > 0 {
            visibility + self.at(x, y - 1).player_visibility
        } else {
            visibility
        }
    }
}

#[test]
fn unit_visibility() {
    use super::units::UnitType;

    let mut tiles = Tiles::new(30, 30);
    let mut units = Units::new();
    units.add(UnitType::Squaddie, UnitSide::Player, 0, 0);
    tiles.update_visibility(&units);

    // A tile a unit is standing on should be visible
    assert_eq!(tiles.at(0, 0).player_visibility, Visibility::Visible);
    // A far away tile should be invisible
    assert_eq!(tiles.at(29, 29).player_visibility, Visibility::Invisible);

    // A tile that was visible but is no longer should be foggy

    units.get_mut(0).unwrap().move_to(29, 0, 0);
    tiles.update_visibility(&units);

    assert_eq!(tiles.at(0, 0).player_visibility, Visibility::Foggy);

    // If the unit is boxed into a corner, only it's tile should be visible

    tiles.add_left_wall(29, 0, WallType::Ruin1);
    tiles.add_top_wall(29, 1, WallType::Ruin2);

    tiles.update_visibility(&units);

    for x in 0 .. tiles.cols {
        for y in 0 .. tiles.rows {
            let visibility = tiles.at(x, y).player_visibility;

            if !(x == 29 && y == 0) {
                assert_ne!(visibility, Visibility::Visible);
            } else {
                assert_eq!(visibility, Visibility::Visible);
            }
        }
    }
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