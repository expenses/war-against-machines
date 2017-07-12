// A UI struct to display clickable buttons and text fields

use resources::{ImageSource, Image};
use context::Context;
use colours::WHITE;

// The vertical alignment of an item
pub enum VerticalAlignment {
    Left,
    Middle,
    Right
}

// The horizontal alignment of an item
pub enum HorizontalAlignment {
    Top,
    Middle,
    Bottom
}

fn get_x(x: f32, width: f32, screen_width: f32, v_align: &VerticalAlignment) -> f32 {
    match *v_align {
        VerticalAlignment::Left => x - (screen_width - width) / 2.0,
        VerticalAlignment::Middle => x,
        VerticalAlignment::Right => x + (screen_width - width) / 2.0
    }
}

fn get_y(y: f32, height: f32, screen_height: f32, h_align: &HorizontalAlignment) -> f32 {
    match *h_align {
        HorizontalAlignment::Top => (screen_height - height) / 2.0 - y,
        HorizontalAlignment::Middle => y,
        HorizontalAlignment::Bottom => -(screen_height - height) / 2.0 - y
    }
}

// A button on the UI
pub struct Button {
    image: Image,
    x: f32,
    y: f32,
    scale: f32,
    active: bool,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment
}

impl Button {
    // Add a new button
    pub fn new(image: Image, x: f32, y: f32, scale: f32, v_align: VerticalAlignment, h_align: HorizontalAlignment) -> Button {     
        Button {
            x, y, scale, v_align, h_align, image,
            active: true
        }
    }

    // Get the width of the button
    fn width(&self) -> f32 {
        self.image.width() * self.scale
    }

    // Get the height of the button
    fn height(&self) -> f32 {
        self.image.height() * self.scale
    }

    // Draw the button at its location and scale
    fn draw(&self, ctx: &mut Context) {
        let x = get_x(self.x, self.width(), ctx.width, &self.v_align);
        let y = get_y(self.y, self.height(), ctx.height, &self.h_align);

        ctx.render(&self.image, x, y, self.scale)
    }

    // Calculate if the button was pressed
    pub fn clicked(&self, ctx: &Context, x: f32, y: f32) -> bool {
        let pos_x = get_x(self.x, self.width(), ctx.width, &self.v_align);
        let pos_y = get_y(self.y, self.height(), ctx.height, &self.h_align);

        x >= pos_x - self.width() / 2.0 && x <= pos_x + self.width() / 2.0 &&
        y >= pos_y - self.width() / 2.0 && y <= pos_y + self.height() / 2.0
    }
}

// A text display on the UI
pub struct TextDisplay {
    pub text: String,
    x: f32,
    y: f32,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment,
    active: bool
}

impl TextDisplay {
    // Create a new text display
    pub fn new(x: f32, y: f32, v_align: VerticalAlignment, h_align: HorizontalAlignment, active: bool) -> TextDisplay {
        TextDisplay {
            x, y, v_align, h_align, active,
            text: "".into()
        }
    }

    // Append a string onto the text display
    pub fn append(&mut self, string: &str) {
        self.text.push_str(&format!("\n{}", string));
    }

    // Draw the text display on the screen
    fn draw(&self, ctx: &mut Context) {
        let height = ctx.font_height() * self.text.lines().count() as f32;
        let mut y = get_y(self.y, height, ctx.height, &self.h_align) + height / 2.0;
        
        for line in self.text.lines() {
            let width = ctx.font_width(line);
            let x = get_x(self.x, width, ctx.width, &self.v_align);

            ctx.render_text(line, x, y, WHITE);
            y -= ctx.font_height();
        }
    }
}

// The UI struct
pub struct UI {
    buttons: Vec<Button>,
    text_displays: Vec<TextDisplay>
}

impl UI {
    // Create a new UI with vecs of buttons and text displays
    pub fn new(buttons: Vec<Button>, text_displays: Vec<TextDisplay>) -> UI {
        UI {
            buttons, text_displays
        }
    }

    // Toggle if a text display is active or not
    pub fn toggle_text_display(&mut self, display: usize) {
        self.text_displays[display].active = !self.text_displays[display].active;
    }

    // Get a mutable reference to a text display
    pub fn ref_mut(&mut self, display: usize) -> &mut TextDisplay {
        &mut self.text_displays[display]
    }

    // Draw all the active buttons and text displays
    pub fn draw(&self, ctx: &mut Context) {
        for button in &self.buttons {
            if button.active {
                button.draw(ctx);
            }
        }

        for text_display in &self.text_displays {
            if text_display.active {
                text_display.draw(ctx);
            }
        }
    }

    // Get the first active clicked button at a location
    pub fn clicked(&self, ctx: &Context, mouse: (f32, f32)) -> Option<usize> {
        self.buttons.iter()
            .enumerate()
            .find(|&(_, button)| button.active && button.clicked(ctx, mouse.0, mouse.1))
            .map(|(i, _)| i)
    }
}