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
    scale: f32
}

impl Button {
    pub fn new(image: usize, x: f32, y: f32, scale: f32, resources: &Resources) -> Button {
        let image_resource = &resources.images[image];
        let width = image_resource.width() as f32 * scale;
        let height = image_resource.height() as f32 * scale;

        Button {
            image, x, y, width, height, scale
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
    text: String
}

impl TextDisplay {
    pub fn new(x: f32, y: f32) -> TextDisplay {
        TextDisplay {
            x,
            y,
            text: String::new()
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
    buttons: Vec<Button>,
    text_displays: Vec<TextDisplay>
}

impl UI {
    pub fn new() -> UI {
        UI {
            buttons: Vec::new(),
            text_displays: Vec::new()
        }
    }

    pub fn add_button(&mut self, button: Button) {
        self.buttons.push(button);
    }

    pub fn add_text_display(&mut self, text: TextDisplay) {
        self.text_displays.push(text);
    }

    pub fn set_text(&mut self, display: usize, text: String) {
        self.text_displays[display].text = text;
    }

    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for button in &self.buttons {
            button.draw(ctx, resources);
        }

        for text_display in &self.text_displays {
            text_display.draw(ctx, resources);
        }
    }

    pub fn clicked(&self, x: f32, y: f32) -> Option<usize> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.clicked(x, y) {
                return Some(i);
            }
        }

        None
    }
}