use rand;
use rand::Rng;

use map::units::{UnitSide, Units};
use utils::distance;

const UNIT_SIGHT: f32 = 7.5;

#[derive(Eq, PartialEq)]
pub enum Visibility {
    Visible,
    Foggy,
    Invisible
}

// A tile in the map
pub struct Tile {
    pub base: String,
    pub decoration: Option<String>,
    pub walkable: bool,
    pub visibility: Visibility
}

impl Tile {
    // Create a new tile
    fn new(base: &str) -> Tile {
        Tile {
            base: base.into(),
            decoration: None,
            walkable: true,
            visibility: Visibility::Invisible
        }
    }

    // Set the decoration of the tile and make it unwalkable
    fn set_obstacle(&mut self, decoration: &str) {
        self.decoration = Some(decoration.into());
        self.walkable = false;        
    }

    pub fn visible(&self) -> bool {
        self.visibility != Visibility::Invisible
    }
}

// A 2D array of tiles
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
    pub fn generate(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;

        let mut rng = rand::thread_rng();
        let ruins = &["ruin_1", "ruin_2", "ruin_3"];
        let bases = &["base_1", "base_2"];

        for _ in 0 .. cols {
            for row in 0 .. rows {
                // Choose a random base image
                let mut tile = Tile::new(*rng.choose(bases).unwrap());

                // Add in skulls
                if rand::random::<f32>() < 0.025 {
                    tile.decoration = Some("skull".into());
                } 

                // Add in ruins
                if row != 0 && row != rows - 1 && rand::random::<f32>() < 0.1 {
                    tile.set_obstacle(*rng.choose(ruins).unwrap());
                }

                // Push the tile
                self.tiles.push(tile);
            }
        }

        // Generate pit position and size
        let mut rng = rand::thread_rng();
        let pit_width = rng.gen_range(1, 4);
        let pit_height = rng.gen_range(1, 4);
        let pit_x = rng.gen_range(1, cols - 1 - pit_width);
        let pit_y = rng.gen_range(1, rows - 1 - pit_height);

        // Add pit corners
        self.tile_at_mut(pit_x,             pit_y             ).set_obstacle("pit_top");
        self.tile_at_mut(pit_x,             pit_y + pit_height).set_obstacle("pit_left");
        self.tile_at_mut(pit_x + pit_width, pit_y             ).set_obstacle("pit_right");
        self.tile_at_mut(pit_x + pit_width, pit_y + pit_height).set_obstacle("pit_bottom");

        // Add pit edges and center

        for x in pit_x + 1 .. pit_x + pit_width {
            self.tile_at_mut(x, pit_y             ).set_obstacle("pit_tr");
            self.tile_at_mut(x, pit_y + pit_height).set_obstacle("pit_bl");

            for y in pit_y + 1 .. pit_y + pit_height {
                self.tile_at_mut(x, y).set_obstacle("pit_center");
            }
        }

        for y in pit_y + 1 .. pit_y + pit_height {
             self.tile_at_mut(pit_x,             y).set_obstacle("pit_tl");
             self.tile_at_mut(pit_x + pit_width, y).set_obstacle("pit_br");
        }
    }

    // Get a reference to a tile
    pub fn tile_at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x * self.rows + y]
    }

    // Get a mutable reference to a tile
    pub fn tile_at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[x * self.rows + y]
    }

    pub fn update_visibility(&mut self, units: &Units) {
        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
                let tile = self.tile_at_mut(x, y);

                let visible = units.iter()
                    .filter(|unit| unit.side == UnitSide::Friendly)
                    .any(|unit| distance(unit.x, unit.y, x, y) <= UNIT_SIGHT);
                
                if visible {
                    tile.visibility = Visibility::Visible;
                } else if tile.visible() {
                    tile.visibility = Visibility::Foggy;
                }
            }
        }
    }
}