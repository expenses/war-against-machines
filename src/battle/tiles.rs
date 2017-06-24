use rand;
use rand::Rng;

use battle::units::{UnitSide, Units};
use utils::distance_under;
use items::{Item, ItemType};

pub const UNIT_SIGHT: f32 = 7.5;

// The visibility of the tile
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum Visibility {
    Visible,
    Foggy,
    Invisible
}

// A tile in the map
pub struct Tile {
    pub base: String,
    pub obstacle: Option<String>,
    pub unit_visibility: Visibility,
    pub enemy_visibility: Visibility,
    pub items: Vec<Item>
}

impl Tile {
    // Create a new tile
    fn new(base: &str) -> Tile {
        Tile {
            base: base.into(),
            obstacle: None,
            unit_visibility: Visibility::Invisible,
            enemy_visibility: Visibility::Invisible,
            items: Vec::new()
        }
    }

    // Set the decoration of the tile and make it unwalkable
    fn set_obstacle(&mut self, decoration: &str) {
        self.obstacle = Some(decoration.into());
        self.items = Vec::new();
    }

    pub fn visible(&self) -> bool {
        self.unit_visibility != Visibility::Invisible
    }

    pub fn walkable(&self) -> bool {
        self.obstacle.is_none()
    }
}

fn visible(x: usize, y: usize, side: UnitSide, units: &Units) -> bool {
    units.iter()
        .filter(|&(_, unit)| unit.side == side)
        .any(|(_, unit)| distance_under(unit.x, unit.y, x, y, UNIT_SIGHT))
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
    pub fn generate(&mut self, cols: usize, rows: usize, units: &Units) {
        self.cols = cols;
        self.rows = rows;

        let mut rng = rand::thread_rng();
        let ruins = &["ruin_1", "ruin_2", "ruin_3"];
        let bases = &["base_1", "base_2"];
        let items = &[ItemType::Weapon, ItemType::Scrap, ItemType::Skeleton];

        for x in 0 .. cols {
            for y in 0 .. rows {
                // Choose a random base image
                let mut tile = Tile::new(*rng.choose(bases).unwrap());

                // Randomly drop items
                if rand::random::<f32>() < 0.075 {
                    tile.items.push(Item::new(*rng.choose(items).unwrap()));
                }

                // Add in ruins
                if units.at(x, y).is_none() && rand::random::<f32>() < 0.1 {
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
        self.at_mut(pit_x,             pit_y             ).set_obstacle("pit_top");
        self.at_mut(pit_x,             pit_y + pit_height).set_obstacle("pit_left");
        self.at_mut(pit_x + pit_width, pit_y             ).set_obstacle("pit_right");
        self.at_mut(pit_x + pit_width, pit_y + pit_height).set_obstacle("pit_bottom");

        // Add pit edges and center

        for x in pit_x + 1 .. pit_x + pit_width {
            self.at_mut(x, pit_y             ).set_obstacle("pit_tr");
            self.at_mut(x, pit_y + pit_height).set_obstacle("pit_bl");

            for y in pit_y + 1 .. pit_y + pit_height {
                self.at_mut(x, y).set_obstacle("pit_center");
            }
        }

        for y in pit_y + 1 .. pit_y + pit_height {
             self.at_mut(pit_x,             y).set_obstacle("pit_tl");
             self.at_mut(pit_x + pit_width, y).set_obstacle("pit_br");
        }

        self.update_visibility(units);
    }

    // Get a reference to a tile
    pub fn at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x * self.rows + y]
    }

    // Get a mutable reference to a tile
    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[x * self.rows + y]
    }

    // Update the visibility of the map
    pub fn update_visibility(&mut self, units: &Units) {
        for x in 0 .. self.cols {
            for y in 0 .. self.rows {
                let tile = self.at_mut(x, y);
                
                if visible(x, y, UnitSide::Friendly, units) {
                    tile.unit_visibility = Visibility::Visible;
                } else if tile.unit_visibility != Visibility::Invisible {
                    tile.unit_visibility = Visibility::Foggy;
                }
                
                if visible(x, y, UnitSide::Enemy, units) {
                    tile.enemy_visibility = Visibility::Visible;
                } else if tile.enemy_visibility != Visibility::Invisible {
                    tile.enemy_visibility = Visibility::Foggy;
                }
            }
        }
    }

    pub fn drop(&mut self, x: usize, y: usize, item: Item) {
        self.at_mut(x, y).items.push(item);
    }
}