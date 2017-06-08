use rand;
use rand::Rng;

use images;
// use items::{Item, ItemType};

pub struct Tile {
    pub base: usize,
    pub decoration: Option<usize>,
    pub walkable: bool,
    // pub items: Vec<Item>
}

impl Tile {
    fn new(base: usize) -> Tile {
        Tile {
            base,
            decoration: None,
            walkable: true,
            // items: Vec::new()
        }
    }

    fn set_pit(&mut self, decoration: usize) {
        self.decoration = Some(decoration);
        self.walkable = false;        
    }
}

pub struct Tiles {
    tiles: Vec<Tile>,
    pub cols: usize,
    pub rows: usize
}

impl Tiles {
    pub fn new(cols: usize, rows: usize) -> Tiles {
        Tiles {
            tiles: Vec::new(),
            cols,
            rows
        }
    }

    pub fn generate(&mut self) {
        for _ in 0 .. self.cols * self.rows {
            let mut tile = Tile::new(if rand::random::<bool>() {
                images::BASE_1
            } else {
                images::BASE_2
            });

            if rand::random::<f32>() > 0.975 {
                tile.decoration = Some(images::SKULL);
            }

            /* if rand::random::<f32>() > 0.99 {
                tile.items.push(Item::new(ItemType::Scrap))
            }

            if rand::random::<f32>() > 0.99 {
                tile.items.push(Item::new(ItemType::Weapon))
            } */

            self.tiles.push(tile);
        }

        // Generate pit position and size
        let mut rng = rand::thread_rng();
        let pit_width = rng.gen_range(1, 4);
        let pit_height = rng.gen_range(1, 4);
        let pit_x = rng.gen_range(1, self.cols - 1 - pit_width);
        let pit_y = rng.gen_range(1, self.rows - 1 - pit_height);

        // Add pit corners
        self.mut_tile_at(pit_x, pit_y).set_pit(images::PIT_TOP);
        self.mut_tile_at(pit_x, pit_y + pit_height).set_pit(images::PIT_LEFT);
        self.mut_tile_at(pit_x + pit_width, pit_y).set_pit(images::PIT_RIGHT);
        self.mut_tile_at(pit_x + pit_width, pit_y + pit_height).set_pit(images::PIT_BOTTOM);

        // Add pit edges and center

        for x in pit_x + 1 .. pit_x + pit_width {
            self.mut_tile_at(x, pit_y).set_pit(images::PIT_TR);
            self.mut_tile_at(x, pit_y + pit_height).set_pit(images::PIT_BL);

            for y in pit_y + 1 .. pit_y + pit_height {
                self.mut_tile_at(x, y).set_pit(images::PIT_CENTER);
            }
        }

        for y in pit_y + 1 .. pit_y + pit_height {
             self.mut_tile_at(pit_x, y).set_pit(images::PIT_TL);
             self.mut_tile_at(pit_x + pit_width, y).set_pit(images::PIT_BR);
        }
    }

    pub fn tile_at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x * self.rows + y]
    }

    fn mut_tile_at(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[x * self.rows + y]
    }
}