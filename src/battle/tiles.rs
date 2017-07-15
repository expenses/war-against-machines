// The tiles in the map, and a struct to contain them

use rand;
use rand::Rng;

use super::units::{UnitSide, Units};
use items::Item;
use resources::Image;

// The visibility of the tile
#[derive(Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Visible,
    Foggy,
    Invisible
}

// A tile in the map
#[derive(Serialize, Deserialize)]
pub struct Tile {
    pub base: Image,
    pub obstacle: Option<Image>,
    pub decoration: Option<Image>,
    pub player_visibility: Visibility,
    pub ai_visibility: Visibility,
    pub items: Vec<Item>
}

impl Tile {
    // Create a new tile
    fn new(base: Image) -> Tile {
        Tile {
            base,
            obstacle: None,
            decoration: None,
            player_visibility: Visibility::Invisible,
            ai_visibility: Visibility::Invisible,
            items: Vec::new()
        }
    }

    // Set the obstacle of the tile and remove the items
    fn set_obstacle(&mut self, decoration: Image) {
        self.obstacle = Some(decoration);
        self.items = Vec::new();
    }

    // return if the tile is visible to the player
    pub fn visible(&self) -> bool {
        self.player_visibility != Visibility::Invisible
    }

    // return if the tile can be walked on
    pub fn walkable(&self) -> bool {
        self.obstacle.is_none()
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
    pub fn new() -> Tiles {
        Tiles {
            tiles: Vec::new(),
            cols: 0,
            rows: 0
        }
    }

    // Generate the tiles
    pub fn generate(&mut self, cols: usize, rows: usize, units: &Units) {
        self.cols = cols;
        self.rows = rows;

        let mut rng = rand::thread_rng();
        let ruins = &[Image::Ruin1, Image::Ruin2, Image::Ruin3, Image::Ruin4];
        let bases = &[Image::Base1, Image::Base2];

        for x in 0 .. cols {
            for y in 0 .. rows {
                // Choose a random base image
                let mut tile = Tile::new(*rng.choose(bases).unwrap());

                // Add in decorations
                if rand::random::<f32>() < 0.05 {
                    tile.decoration = Some(if rand::random::<bool>() {
                        Image::Skeleton
                    } else {
                        Image::Rubble
                    });
                }

                // Add in ruins
                if units.at(x, y).is_none() && rand::random::<f32>() < 0.1 {
                    tile.set_obstacle(*rng.choose(ruins).unwrap());
                } 

                // Push the tile
                self.tiles.push(tile);
            }
        }

        // Generate a randomly sized pit
        let mut rng = rand::thread_rng();        
        self.add_pit(rng.gen_range(2, 5), rng.gen_range(2, 5));

        self.update_visibility(units);
    }

    fn add_pit(&mut self, width: usize, height: usize) {
        // Generate pit position and size
        let mut rng = rand::thread_rng();
        let pit_x = rng.gen_range(1, self.cols - width  - 1);
        let pit_y = rng.gen_range(1, self.rows - height - 1);

        // Add pit corners
        self.at_mut(pit_x,             pit_y             ).set_obstacle(Image::PitTop);
        self.at_mut(pit_x,             pit_y + height - 1).set_obstacle(Image::PitLeft);
        self.at_mut(pit_x + width - 1, pit_y             ).set_obstacle(Image::PitRight);
        self.at_mut(pit_x + width - 1, pit_y + height - 1).set_obstacle(Image::PitBottom);

        // Add pit edges and center
        for x in pit_x + 1 .. pit_x + width - 1 {
            self.at_mut(x, pit_y             ).set_obstacle(Image::PitTR);
            self.at_mut(x, pit_y + height - 1).set_obstacle(Image::PitBL);

            for y in pit_y + 1 .. pit_y + height - 1 {
                self.at_mut(x, y).set_obstacle(Image::PitCenter);
            }
        }

        for y in pit_y + 1 .. pit_y + height - 1 {
            self.at_mut(pit_x,             y).set_obstacle(Image::PitTL);
            self.at_mut(pit_x + width - 1, y).set_obstacle(Image::PitBR);
        }
    }

    // Get a reference to a tile
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        self.tiles.get(x * self.rows + y)
            .unwrap_or_else(|| panic!("Tile at ({}, {}) out of bounds", x, y))
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        self.tiles.get_mut(x * self.rows + y)
            .unwrap_or_else(|| panic!("Tile at ({}, {}) out of bounds", x, y))
    }

    // Update the visibility of the map
    pub fn update_visibility(&mut self, units: &Units) {
        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
                let tile = self.at_mut(x, y);
                
                if units.visible(x, y, UnitSide::Player) {
                    tile.player_visibility = Visibility::Visible;
                } else if tile.player_visibility != Visibility::Invisible {
                    tile.player_visibility = Visibility::Foggy;
                }
                
                if units.visible(x, y, UnitSide::AI) {
                    tile.ai_visibility = Visibility::Visible;
                } else if tile.ai_visibility != Visibility::Invisible {
                    tile.ai_visibility = Visibility::Foggy;
                }
            }
        }
    }

    // Drop an item onto the map
    pub fn drop(&mut self, x: usize, y: usize, item: Item) {
        self.at_mut(x, y).items.push(item);
    }

    pub fn drop_all(&mut self, x: usize, y: usize, items: &mut Vec<Item>) {
        self.at_mut(x, y).items.append(items);
    }
}