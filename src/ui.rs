// A UI struct to display clickable buttons and text fields

use graphics::{Context, ImageSize, Transformed, image, DrawState};
use graphics::character::CharacterCache;
use utils::Dimensions;
use opengl_graphics::GlGraphics;

use Resources;
use constants::{WHITE, REGULAR};

// The vertical alignment of an item
pub enum VerticalAlignment {
    _Left,
    Middle,
    Right
}

// The horizontal alignment of an item
pub enum HorizontalAlignment {
    Top,
    Middle,
    Bottom
}

fn get_location(x: f64, y: f64, width: f64, height: f64, v_align: &VerticalAlignment, h_align: &HorizontalAlignment, ctx: &Context) -> (f64, f64) {
    let (screen_width, screen_height) = (ctx.get_width(), ctx.get_height());

    let x = match *v_align {
        VerticalAlignment::_Left => x,
        VerticalAlignment::Middle => (screen_width - width)  / 2.0 + x,
        VerticalAlignment::Right => (screen_width - width) + x
    };

    let y = match *h_align {
        HorizontalAlignment::Top => y,
        HorizontalAlignment::Middle => (screen_height - height) / 2.0 + y,
        HorizontalAlignment::Bottom => (screen_height - height) + y
    };

    (x, y)
}

// A button on the UI
pub struct Button {
    image: &'static str,
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
    pub fn new(image: &'static str, x: f64, y: f64, scale: f64, resources: &Resources,
               v_align: VerticalAlignment, h_align: HorizontalAlignment) -> Button {
        let image_resource = resources.image(image);
        
        let width = image_resource.get_width() as f64 * scale;
        let height = image_resource.get_height() as f64 * scale;

        Button {
            x, y, width, height, scale, v_align, h_align, image,
            active: true
        }
    }

    // Draw the button at its location and scale
    fn draw(&self, ctx: &Context, gl: &mut GlGraphics, resources: &Resources) {
        let (x, y) = get_location(self.x, self.y, self.width, self.height, &self.v_align, &self.h_align, ctx);

        image(resources.image(self.image), ctx.transform.trans(x, y).scale(self.scale, self.scale), gl);
    }

    // Calculate if the button was pressed
    pub fn clicked(&self, ctx: &Context, x: f64, y: f64) -> bool {
        let (pos_x, pos_y) = get_location(self.x, self.y, self.width, self.height, &self.v_align, &self.h_align, ctx);

        x >= pos_x && x <= pos_x + self.width &&
        y >= pos_y && y <= pos_y + self.height
    }
}

// A text display on the UI
pub struct TextDisplay {
    x: f64,
    y: f64,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment,
    text: String,
    active: bool
}

impl TextDisplay {
    // Create a new text display
    pub fn new(x: f64, y: f64, v_align: VerticalAlignment, h_align: HorizontalAlignment, active: bool, text: &str) -> TextDisplay {
        TextDisplay {
            x, y, v_align, h_align, active,
            text: text.into()
        }
    }

    // Draw the text display on the screen
    fn draw(&self, ctx: &Context, gl: &mut GlGraphics, resources: &mut Resources) {
        let mut y_offset = 0.0;

        // Iterate through the non-empty lines
        for line in self.text.split('\n').filter(|line| !line.is_empty()) {
            let width = resources.font.width(REGULAR.font_size, &line);
            let height = REGULAR.font_size as f64;

            y_offset += height;

            let (x, y) = get_location(self.x, self.y, width, height, &self.v_align, &self.h_align, ctx);

            let rendered = REGULAR.draw(
                &line,
                &mut resources.font,
                &DrawState::default(),
                ctx.transform.trans(x, y + y_offset),
                gl
            );
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

    // Set the text of a text display
    pub fn set_text(&mut self, display: usize, text: String) {
        self.text_displays[display].text = text;
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
    pub fn clicked(&self, ctx: &Context, x: f64, y: f64) -> Option<usize> {
        self.buttons.iter()
            .enumerate()
            .find(|&(_, button)| button.active && button.clicked(ctx, x, y))
            .map(|(i, _)| i)
    }
}