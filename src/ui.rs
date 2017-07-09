// A UI struct to display clickable buttons and text fields

use graphics::{Context, Transformed};
use traits::Dimensions;
use opengl_graphics::GlGraphics;

use resources::{Resources, SetImage};
use WindowSize;

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

fn get_x(x: f64, width: f64, screen_width: f64, v_align: &VerticalAlignment) -> f64 {
    match *v_align {
        VerticalAlignment::Left => x,
        VerticalAlignment::Middle => (screen_width - width) / 2.0 + x,
        VerticalAlignment::Right => (screen_width - width) + x
    }
}

fn get_y(y: f64, height: f64, screen_height: f64, h_align: &HorizontalAlignment) -> f64 {
    match *h_align {
        HorizontalAlignment::Top => y,
        HorizontalAlignment::Middle => (screen_height - height) / 2.0 + y,
        HorizontalAlignment::Bottom => (screen_height - height) + y
    }
}

// A button on the UI
pub struct Button {
    image: SetImage,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    scale: f64,
    active: bool,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment
}

impl Button {
    // Add a new button
    pub fn new(image: SetImage, x: f64, y: f64, scale: f64, v_align: VerticalAlignment, h_align: HorizontalAlignment) -> Button {     
        let width = image.width() * scale;
        let height = image.height() * scale;

        Button {
            x, y, width, height, scale, v_align, h_align, image,
            active: true
        }
    }

    // Draw the button at its location and scale
    fn draw(&self, ctx: &Context, gl: &mut GlGraphics, resources: &Resources) {
        let x = get_x(self.x, self.width, ctx.width(), &self.v_align);
        let y = get_y(self.y, self.height, ctx.height(), &self.h_align);

        resources.render(&self.image, ctx.transform.trans(x, y).scale(self.scale, self.scale), gl)
    }

    // Calculate if the button was pressed
    pub fn clicked(&self, window_size: &WindowSize, x: f64, y: f64) -> bool {
        let pos_x = get_x(self.x, self.width, window_size.width, &self.v_align);
        let pos_y = get_y(self.y, self.height, window_size.height, &self.h_align);

        x >= pos_x && x <= pos_x + self.width &&
        y >= pos_y && y <= pos_y + self.height
    }
}

// A text display on the UI
pub struct TextDisplay {
    pub text: String,
    x: f64,
    y: f64,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment,
    active: bool
}

impl TextDisplay {
    // Create a new text display
    pub fn new(x: f64, y: f64, v_align: VerticalAlignment, h_align: HorizontalAlignment, active: bool) -> TextDisplay {
        TextDisplay {
            x, y, v_align, h_align, active,
            text: "".into()
        }
    }

    pub fn append(&mut self, string: &str) {
        self.text.push_str(&format!("\n{}", string));
    }

    // Draw the text display on the screen
    fn draw(&self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        let height = resources.font_height() * self.text.lines().count() as f64;
        let mut y = get_y(self.y, height, ctx.height(), &self.h_align);
        
        for line in self.text.lines() {
            let width = resources.font_width(line);
            let x = get_x(self.x, width, ctx.width(), &self.v_align);

            resources.render_text(line, ctx.transform.trans(x, y), gl);
            y += resources.font_height();
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
    pub fn draw(&self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        for button in &self.buttons {
            if button.active {
                button.draw(ctx, gl, resources);
            }
        }

        for text_display in &self.text_displays {
            if text_display.active {
                text_display.draw(ctx, gl, resources);
            }
        }
    }

    // Get the first active clicked button at a location
    pub fn clicked(&self, window_size: &WindowSize, mouse: (f64, f64)) -> Option<usize> {
        self.buttons.iter()
            .enumerate()
            .find(|&(_, button)| button.active && button.clicked(window_size, mouse.0, mouse.1))
            .map(|(i, _)| i)
    }
}