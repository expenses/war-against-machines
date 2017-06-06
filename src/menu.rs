use ggez::Context;
use ggez::graphics;
use ggez::event::Keycode;
use ggez::graphics::{Font, Image, Point, Text};

use images;
use WINDOW_WIDTH;

const OPTIONS: [&str; 2] = [
    "Play Game",
    "Quit"
];

pub struct Menu {
    selection: usize
}

impl Menu {
    pub fn new() -> Menu {
        Menu {
            selection: 0
        }
    }

    pub fn draw(&self, ctx: &mut Context, images: &Vec<Image>, font: &Font) {
        let ref title = images[images::TITLE];

        let title_x = WINDOW_WIDTH as f32 / 2.0;
        let title_y = title.height() as f32 / 2.0;

        graphics::draw(ctx, title, Point{x: title_x, y: title_y}, 0.0).unwrap();

        for (i, option) in OPTIONS.iter().enumerate() {
            let mut string = String::new();

            if i == self.selection {
                string.push_str("> ");
            }

            string.push_str(option);

            let rendered = Text::new(ctx, string.as_str(), font).unwrap();

            let point = Point {
                x: title_x,
                y: title_y + 50.0 + i as f32 * 20.0
            };

            graphics::draw(ctx, &rendered, point, 0.0).unwrap();
        }
    }

    pub fn handle_key(&mut self, key: Keycode) -> Option<usize> {
        match key {
            Keycode::Up => self.selection = (self.selection + 1) % 2,
            Keycode::Down => self.selection = match self.selection {
                0 => 1,
                selection => selection - 1
            },
            Keycode::Return => return Some(self.selection),
            _ => {}
        }

        None
    }
}