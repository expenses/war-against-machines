use ggez::Context;
use ggez::graphics;
use ggez::graphics::{Point, DrawParam, Text};

use Resources;

pub struct Button {
    image: usize,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    scale: f32,
    active: bool,
}

impl Button {
    pub fn new(image: usize, x: f32, y: f32, scale: f32, resources: &Resources) -> Button {
        let image_resource = &resources.images[image];
        let width = image_resource.width() as f32 * scale;
        let height = image_resource.height() as f32 * scale;

        Button {
            image, x, y, width, height, scale,
            active: true
        }
    }

    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        graphics::draw_ex(
            ctx,
            &resources.images[self.image],
            DrawParam {
                dest: Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0),
                scale: Point::new(self.scale, self.scale),
                ..Default::default()
            }
        ).unwrap();
    }

    fn clicked(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

pub struct TextDisplay {
    x: f32,
    y: f32,
    text: String,
    active: bool
}

impl TextDisplay {
    pub fn new(x: f32, y: f32) -> TextDisplay {
        TextDisplay {
            x, y,
            text: String::new(),
            active: true
        }
    }

    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        let rendered = Text::new(ctx, self.text.as_str(), &resources.font).unwrap();
        let point = Point::new(
            self.x + rendered.width() as f32 / 2.0,
            self.y + rendered.height() as f32 / 2.0
        );

        graphics::draw(ctx, &rendered, point, 0.0).unwrap();
    }
}

pub struct UI {
    pub buttons: Vec<Button>,
    pub text_displays: Vec<TextDisplay>
}

impl UI {
    pub fn new() -> UI {
        UI {
            buttons: Vec::new(),
            text_displays: Vec::new()
        }
    }

    pub fn set_text(&mut self, display: usize, text: String) {
        self.text_displays[display].text = text;
    }

    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for button in &self.buttons {
            if button.active {
                button.draw(ctx, resources);
            }
        }

        for text_display in &self.text_displays {
            if text_display.active {
                text_display.draw(ctx, resources);
            }
        }
    }

    pub fn clicked(&self, x: f32, y: f32) -> Option<usize> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.active && button.clicked(x, y) {
                return Some(i);
            }
        }

        None
    }
}