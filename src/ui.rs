//! A UI struct to display clickable buttons and text fields

use Resources;
use context::Context;
use colours::WHITE;

/// The vertical alignment of an item
pub enum VerticalAlignment {
    Left,
    Middle,
    Right
}

/// The horizontal alignment of an item
pub enum HorizontalAlignment {
    Top,
    Middle,
    Bottom
}

/// A button on the UI
pub struct Button {
    image: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    scale: f32,
    active: bool,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment
}

impl Button {
    /// Add a new button
    pub fn new(image: &str, x: f32, y: f32, scale: f32, resources: &Resources,
               v_align: VerticalAlignment, h_align: HorizontalAlignment) -> Button {
        let image_resource = resources.image(image);
        
        let query = image_resource.query();
        let width = query.width as f32 * scale;
        let height = query.height as f32 * scale;

        Button {
            x, y, width, height, scale, v_align, h_align,
            active: true,
            image: image.into()
        }
    }

    /// Get the location of the button
    pub fn get_location(&self, ctx: &mut Context) -> (f32, f32) {
        let (width, height) = (ctx.width() as f32, ctx.height() as f32);

        let x = match self.v_align {
            VerticalAlignment::Left => self.x,
            VerticalAlignment::Middle => (width - self.width)  / 2.0 + self.x,
            VerticalAlignment::Right => (width - self.width) + self.x
        };

        let y = match self.h_align {
            HorizontalAlignment::Top => self.y,
            HorizontalAlignment::Middle => (height - self.height) / 2.0 + self.y,
            HorizontalAlignment::Bottom => (height - self.height) + self.y
        };

        (x, y)
    }

    /// Draw the button at its location and scale
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        let (x, y) = self.get_location(ctx);

        ctx.draw(resources.image(&self.image), x, y, self.scale);
    }

    /// Calculate if the button was pressed
    pub fn clicked(&self, ctx: &mut Context, x: f32, y: f32) -> bool {
        let (pos_x, pos_y) = self.get_location(ctx);

        x >= pos_x && x <= pos_x + self.width &&
        y >= pos_y && y <= pos_y + self.height
    }
}

/// A text display on the UI
pub struct TextDisplay {
    x: f32,
    y: f32,
    v_align: VerticalAlignment,
    h_align: HorizontalAlignment,
    text: String,
    active: bool
}

impl TextDisplay {
    /// Create a new text display
    pub fn new(x: f32, y: f32, v_align: VerticalAlignment, h_align: HorizontalAlignment, active: bool, text: &str) -> TextDisplay {
        TextDisplay {
            x, y, v_align, h_align, active,
            text: text.into()
        }
    }

    /// Draw the text display on the screen
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        let rendered = resources.render("main", &self.text, WHITE);
        let query = rendered.query();
        let (width, height) = (query.width as f32, query.height as f32);
        let (screen_width, screen_height) = (ctx.width() as f32, ctx.height() as f32);

        let x = match self.v_align {
            VerticalAlignment::Left => self.x,
            VerticalAlignment::Middle => (screen_width - width)  / 2.0 + self.x,
            VerticalAlignment::Right => (screen_width - width) + self.x
        };

        let y = match self.h_align {
            HorizontalAlignment::Top => self.y,
            HorizontalAlignment::Middle => (screen_height - height) / 2.0 + self.y,
            HorizontalAlignment::Bottom => (screen_height - height) + self.y
        };

        ctx.draw(&rendered, x, y, 1.0);
    }
}

/// The UI struct
pub struct UI {
    pub buttons: Vec<Button>,
    pub text_displays: Vec<TextDisplay>
}

impl UI {
    /// Create a new `UI` with vecs of buttons and text displays
    pub fn new(buttons: Vec<Button>, text_displays: Vec<TextDisplay>) -> UI {
        UI {
            buttons, text_displays
        }
    }

    /// Toggle if a text display is active or not
    pub fn toggle_text_display(&mut self, display: usize) {
        self.text_displays[display].active = !self.text_displays[display].active;
    }

    /// Set the text of a text display
    pub fn set_text(&mut self, display: usize, text: String) {
        self.text_displays[display].text = text;
    }

    /// Draw all the active buttons and text displays
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

    /// Get the first active clicked button at a location
    pub fn clicked(&self, ctx: &mut Context, x: f32, y: f32) -> Option<usize> {
        self.buttons.iter()
            .enumerate()
            .find(|&(_, button)| button.active && button.clicked(ctx, x, y))
            .map(|(i, _)| i)
    }
}