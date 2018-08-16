// The tiles in the map, and a struct to contain them

use rand;
use rand::Rng;

use super::super::units::*;
use super::walls::*;
use super::iter_2d::Iter2D;

use super::grid::*;
use items::Item;
use utils::*;
use resources::Image;

// todo: make map generation better!
const MIN_PIT_SIZE: usize = 2;
const MAX_PIT_SIZE: usize = 5;

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
    pub fn colour(self, light: f32, debug: bool) -> [f32; 4] {
        // The rate at which tiles get darker
        let rate = lerp(Self::NIGHT_DARKNESS_RATE, Self::DAY_DARKNESS_RATE, light);

        let alpha = match self {
            Visibility::Visible(distance) => {
                if debug {
                    return [1.0, 0.0, 0.0, 0.25];
                }

                f32::from(distance) * rate
            },
            Visibility::Foggy => {
                if debug {
                    return [0.0, 1.0, 0.0, 0.25];
                }

                rate * (Unit::SIGHT * f32::from(Unit::WALK_LATERAL_COST)) + 0.1
            }
            Visibility::Invisible => {
                if debug {
                    return [0.0, 0.0, 1.0, 0.25];
                }
                1.0
            }
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

#[derive(Serialize, Deserialize, is_enum_variant, Debug, Clone, PartialEq)]
pub enum Obstacle {
    Object(Image),
    Pit(Image),
    Empty
}

// A tile in the map
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tile {
    pub base: Image,
    pub obstacle: Obstacle,
    pub decoration: Option<Image>,
    pub walls: Walls,
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
            items: Vec::new()
        }
    }

    // Set the tile to be one of the pit images and remove the decoration
    fn set_pit(&mut self, pit_image: Image) {
        self.obstacle = Obstacle::Pit(pit_image);
        self.decoration = None;
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tiles {
    tiles: Grid<Tile>,
    visibility_grids: [Grid<Visibility>; 2]
}

impl Tiles {
    // Create a new set of tiles but do not generate it
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let bases = &[Image::Base1, Image::Base2];
        let tiles = Grid::new(width, height, || Tile::new(*rng.choose(bases).unwrap()));

        Self {
            tiles, visibility_grids: [Grid::new(width, height, || Visibility::Invisible), Grid::new(width, height, || Visibility::Invisible)]
        }
    }

    pub fn visibility_at(&self, x: usize, y: usize, side: Side) -> Visibility {
        match side {
            Side::PlayerA => *self.visibility_grids[0].at(x, y),
            Side::PlayerB => *self.visibility_grids[1].at(x, y)
        }
    }

    pub fn set_visibility_at(&mut self, x: usize, y: usize, side: Side, visibility: Visibility) {
        match side {
            Side::PlayerA => *self.visibility_grids[0].at_mut(x, y) = visibility,
            Side::PlayerB => *self.visibility_grids[1].at_mut(x, y) = visibility
        }
    }

    pub fn width(&self) -> usize {
        self.tiles.width()
    }

    pub fn height(&self) -> usize {
        self.tiles.height()
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
        if self.tiles.in_bounds(x, y) && (self.not_pit(x, y) || self.not_pit(x - 1, y)) {
            self.at_mut(x, y).walls.set_left(tag);
        }
    }

    // Add a top wall if possible
    pub fn add_top_wall(&mut self, x: usize, y: usize, tag: WallType) {
        if self.tiles.in_bounds(x, y) && (self.not_pit(x, y) || self.not_pit(x, y - 1)) {
            self.at_mut(x, y).walls.set_top(tag);
        }
    }

    // Check if a position is in-bounds and not a pit
    fn not_pit(&self, x: usize, y: usize) -> bool {
        self.tiles.in_bounds(x, y) && !self.at(x, y).obstacle.is_pit()
    }

    fn add_pit(&mut self, width: usize, height: usize) {
        // Generate pit position and size
        let mut rng = rand::thread_rng();

        let max_x = self.width() - width  - 1;
        let max_y = self.height() - height - 1;

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
        self.tiles.at(x, y)
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        self.tiles.at_mut(x, y)
    }

    // Update the visibility of the map
    pub fn update_visibility(&mut self, units: &Units) {
        for (x, y) in self.iter() {
            let player_a_visible = self.tile_visible(units, Side::PlayerA, x, y);
            let player_b_visible = self.tile_visible(units, Side::PlayerB, x, y);
            
            let player_a_already_visible = self.visibility_at(x, y, Side::PlayerA).is_visible();
            let player_b_already_visible = self.visibility_at(x, y, Side::PlayerB).is_visible();

            if let Some(distance) = player_a_visible {
                self.set_visibility_at(x, y, Side::PlayerA, Visibility::Visible(distance));
            } else if player_a_already_visible {
                self.set_visibility_at(x, y, Side::PlayerA, Visibility::Foggy);
            }
            
            if let Some(distance) = player_b_visible {
                self.set_visibility_at(x, y, Side::PlayerB, Visibility::Visible(distance));
            } else if player_b_already_visible {
                self.set_visibility_at(x, y, Side::PlayerB, Visibility::Foggy);
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

    // Is a tile visible by any unit on a particular side
    fn tile_visible(&self, units: &Units, side: Side, x: usize, y: usize) -> Option<u8> {
        units.iter()
            .filter(|unit| unit.side == side)
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
        if x.wrapping_sub(1) >= self.width() - 1 || y.wrapping_sub(1) >= self.height() - 1 {
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
    pub fn left_wall_visibility(&self, x: usize, y: usize, side: Side) -> Visibility {
        let visibility = self.visibility_at(x, y, side);

        if x > 0 {
            combine_visibilities(visibility, self.visibility_at(x - 1, y, side))
        } else {
            visibility
        }
    }

    // What should the visibility of a top wall at a position be
    pub fn top_wall_visibility(&self, x: usize, y: usize, side: Side) -> Visibility {
        let visibility = self.visibility_at(x, y, side);
        
        if y > 0 {
            combine_visibilities(visibility, self.visibility_at(x, y - 1, side))
        } else {
            visibility
        }
    }

    // Iterate through the rows and columns
    pub fn iter(&self) -> Iter2D {
        Iter2D::new(self.width(), self.height())
    }

    pub fn visible_units<'a>(&'a self, units: &'a Units, side: Side) -> impl Iterator<Item=&'a Unit> {
        units.iter().filter(move |unit| self.visibility_at(unit.x, unit.y, side).is_visible())
    }

    pub fn clone_visible(&self, side: Side) -> Self {
        let mut tiles = self.tiles.clone();

        for (x, y) in self.iter() {
            // Wipe the info of tiles that are not visible
            if !self.visibility_at(x, y, side).is_visible() {
                let walls = tiles.at(x, y).walls.clone();
                
                let tile = tiles.at_mut(x, y);
                *tile = Tile::new(Image::Base1);
                
                if self.left_wall_visibility(x, y, side).is_visible() {
                    tile.walls.left = walls.left.clone();
                }

                if self.top_wall_visibility(x, y, side).is_visible() {
                    tile.walls.top = walls.top.clone();
                }
            }
        }

        let grids = if side == Side::PlayerA {
            [self.visibility_grids[0].clone(), Grid::new(0, 0, || Visibility::Invisible)]
        } else {
            [Grid::new(0, 0, || Visibility::Invisible), self.visibility_grids[1].clone()]
        };

        Self {
            tiles,
            visibility_grids: grids
        }
    }

    pub fn update_from(&mut self, mut new: Self, side: Side) {
        for (x, y) in self.iter() {
            // Replace foggy tiles with what they look like currently to make sure no infomation is lost
            if new.visibility_at(x, y, side).is_foggy() {
                *new.at_mut(x, y) = self.at(x, y).clone();
            }
        }

        *self = new;
    }
}

#[test]
fn pit_generation() {
    let mut tiles = Tiles::new(30, 30);
    tiles.generate(&Units::new());

    // At least one tile should have a pit on it
    assert!(tiles.iter().map(|(x, y)| tiles.at(x, y)).any(|tile| tile.obstacle.is_pit()));
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