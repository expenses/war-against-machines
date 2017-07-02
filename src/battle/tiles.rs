// The tiles in the map, and a struct to contain them

use rand;
use rand::Rng;

use battle::units::{UnitSide, Units};
use items::{Item, ItemType};

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
    pub base: String,
    pub obstacle: Option<String>,
    pub player_visibility: Visibility,
    pub ai_visibility: Visibility,
    pub items: Vec<Item>
}

impl Tile {
    // Create a new tile
    fn new(base: &str) -> Tile {
        Tile {
            base: base.into(),
            obstacle: None,
            player_visibility: Visibility::Invisible,
            ai_visibility: Visibility::Invisible,
            items: Vec::new()
        }
    }

    // Set the obstacle of the tile and remove the units
    fn set_obstacle(&mut self, decoration: &str) {
        self.obstacle = Some(decoration.into());
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
        let ruins = &["ruin_1", "ruin_2", "ruin_3"];
        let bases = &["base_1", "base_2"];

        for x in 0 .. cols {
            for y in 0 .. rows {
                // Choose a random base image
                let mut tile = Tile::new(*rng.choose(bases).unwrap());

                // Randomly drop items
                if rand::random::<f32>() < 0.025 {
                    tile.items.push(Item::new(ItemType::Skeleton));
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
        self.at_mut(pit_x,             pit_y             ).set_obstacle("pit_top");
        self.at_mut(pit_x,             pit_y + height - 1).set_obstacle("pit_left");
        self.at_mut(pit_x + width - 1, pit_y             ).set_obstacle("pit_right");
        self.at_mut(pit_x + width - 1, pit_y + height - 1).set_obstacle("pit_bottom");

        // Add pit edges and center
        for x in pit_x + 1 .. pit_x + width - 1 {
            self.at_mut(x, pit_y             ).set_obstacle("pit_tr");
            self.at_mut(x, pit_y + height - 1).set_obstacle("pit_bl");

            for y in pit_y + 1 .. pit_y + height - 1 {
                self.at_mut(x, y).set_obstacle("pit_center");
            }
        }

        for y in pit_y + 1 .. pit_y + height - 1 {
            self.at_mut(pit_x,             y).set_obstacle("pit_tl");
            self.at_mut(pit_x + width - 1, y).set_obstacle("pit_br");
        }
    }

    // Get a reference to a tile
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        self.tiles.get(x * self.rows + y)
            .expect(&format!("Tile at ({}, {}) out of bounds for Tiles of size ({}, {})", x, y, self.cols, self.rows))
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        self.tiles.get_mut(x * self.rows + y)
            .expect(&format!("Tile at ({}, {}) out of bounds for Tiles of size ({}, {})", x, y, self.cols, self.rows))
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