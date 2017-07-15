// A UI struct to display clickable buttons and text fields

use glutin::VirtualKeyCode;

use resources::{ImageSource, Image, ToChar};
use context::Context;
use colours::WHITE;

// The vertical alignment of an item
#[derive(Clone, Copy)]
pub enum Vertical {
    Left,
    Middle,
    Right
}

// The horizontal alignment of an item
#[derive(Clone, Copy)]
pub enum Horizontal {
    Top,
    Middle,
    Bottom
}

fn get_x(x: f32, width: f32, screen_width: f32, v_align: &Vertical) -> f32 {
    match *v_align {
        Vertical::Left => x - (screen_width - width) / 2.0,
        Vertical::Middle => x,
        Vertical::Right => x + (screen_width - width) / 2.0
    }
}

fn get_y(y: f32, height: f32, screen_height: f32, h_align: &Horizontal) -> f32 {
    match *h_align {
        Horizontal::Top => (screen_height - height) / 2.0 - y,
        Horizontal::Middle => y,
        Horizontal::Bottom => -(screen_height - height) / 2.0 - y
    }
}

// A button on the UI
pub struct Button {
    image: Image,
    x: f32,
    y: f32,
    scale: f32,
    active: bool,
    v_align: Vertical,
    h_align: Horizontal
}

impl Button {
    // Add a new button
    pub fn new(image: Image, x: f32, y: f32, scale: f32, v_align: Vertical, h_align: Horizontal) -> Button {     
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

        ctx.render(&self.image, [x, y], self.scale)
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
    v_align: Vertical,
    h_align: Horizontal,
    active: bool
}

impl TextDisplay {
    // Create a new text display
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool) -> TextDisplay {
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

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }
}

pub struct TextInput {
    title: TextDisplay,
    input: TextDisplay,
    pub active: bool
}

impl TextInput {
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool, ctx: &Context, display: &str) -> TextInput {
        let mut title = TextDisplay::new(x, y, v_align, h_align, active);
        title.text = display.into();

        TextInput {
            title, active,
            input: TextDisplay::new(x, y - ctx.font_height(), v_align, h_align, active)
        }
    }

    fn draw(&self, ctx: &mut Context) {
        self.title.draw(ctx);
        self.input.draw(ctx);
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
        self.title.active = self.active;
        self.input.active = self.active;
    }

    pub fn handle_key(&mut self, key: VirtualKeyCode) {
        if key == VirtualKeyCode::Back {
            self.input.text.pop();
        } else {
            self.input.text.push(match key.to_char() {
                '�' => return,
                character => character
            });
        }
    }

    pub fn text(&self) -> String {
        self.input.text.clone()
    }
}

// The UI struct
pub struct UI {
    buttons: Vec<Button>,
    text_displays: Vec<TextDisplay>,
    text_inputs: Vec<TextInput>
}

impl UI {
    // Create a new UI with vecs of buttons and text displays
    pub fn new(buttons: Vec<Button>, text_displays: Vec<TextDisplay>, text_inputs: Vec<TextInput>) -> UI {
        UI {
            buttons, text_displays, text_inputs
        }
    }

    // Get a mutable reference to a text display
    pub fn text_display(&mut self, display: usize) -> &mut TextDisplay {
        &mut self.text_displays[display]
    }

    pub fn text_input(&mut self, input: usize) -> &mut TextInput {
        &mut self.text_inputs[input]
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

        for text_input in &self.text_inputs {
            if text_input.active {
                text_input.draw(ctx);
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