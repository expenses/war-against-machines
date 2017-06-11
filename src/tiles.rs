use rand;
use rand::Rng;

use images;

pub struct Tile {
    pub base: usize,
    pub decoration: Option<usize>,
    pub walkable: bool,
}

impl Tile {
    fn new(base: usize) -> Tile {
        Tile {
            base,
            decoration: None,
            walkable: true,
        }
    }

    fn set_obstacle(&mut self, decoration: usize) {
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
    pub fn new() -> Tiles {
        Tiles {
            tiles: Vec::new(),
            cols: 0,
            rows: 0
        }
    }

    pub fn generate(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;

        for _ in 0 .. cols {
            for row in 0 .. rows {
                let mut tile = Tile::new(if rand::random::<bool>() {
                    images::BASE_1
                } else {
                    images::BASE_2
                });

                // Add in skulls
                if rand::random::<f32>() > 0.975 {
                    tile.decoration = Some(images::SKULL);
                } 

                // Add in ruins
                if row != 0 && row != rows - 1 && rand::random::<f32>() > 0.90 {
                    if rand::random::<bool>() {
                        tile.set_obstacle(images::RUIN_1);
                    } else {
                        tile.set_obstacle(images::RUIN_2);
                    }
                }

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
        self.tile_at_mut(pit_x,             pit_y             ).set_obstacle(images::PIT_TOP);
        self.tile_at_mut(pit_x,             pit_y + pit_height).set_obstacle(images::PIT_LEFT);
        self.tile_at_mut(pit_x + pit_width, pit_y             ).set_obstacle(images::PIT_RIGHT);
        self.tile_at_mut(pit_x + pit_width, pit_y + pit_height).set_obstacle(images::PIT_BOTTOM);

        // Add pit edges and center

        for x in pit_x + 1 .. pit_x + pit_width {
            self.tile_at_mut(x, pit_y             ).set_obstacle(images::PIT_TR);
            self.tile_at_mut(x, pit_y + pit_height).set_obstacle(images::PIT_BL);

            for y in pit_y + 1 .. pit_y + pit_height {
                self.tile_at_mut(x, y).set_obstacle(images::PIT_CENTER);
            }
        }

        for y in pit_y + 1 .. pit_y + pit_height {
             self.tile_at_mut(pit_x,             y).set_obstacle(images::PIT_TL);
             self.tile_at_mut(pit_x + pit_width, y).set_obstacle(images::PIT_BR);
        }
    }

    pub fn tile_at(&self, x: usize, y: usize) -> &Tile {
        &self.tiles[x * self.rows + y]
    }

    fn tile_at_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        &mut self.tiles[x * self.rows + y]
    }
}