use Resources;
use context::Context;

pub enum VerticalAlignment {
    _Left,
    _Middle,
    Right
}

pub enum HorizontalAlignment {
    _Top,
    _Middle,
    Bottom
}

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
    pub fn new(image: String, x: f32, y: f32, scale: f32, resources: &Resources, v_align: VerticalAlignment, h_align: HorizontalAlignment) -> Button {
        let image_resource = resources.image(&image);
        
        let query = image_resource.query();
        let width = query.width as f32 * scale;
        let height = query.height as f32 * scale;

        Button {
            image, x, y, width, height, scale, v_align, h_align,
            active: true
        }
    }

    fn get_location(&self, ctx: &mut Context) -> (f32, f32) {
        let (width, height) = (ctx.width() as f32, ctx.height() as f32);

        let x = match self.v_align {
            VerticalAlignment::_Left => self.x,
            VerticalAlignment::_Middle => (width - self.width)  / 2.0 + self.x,
            VerticalAlignment::Right => (width - self.width) + self.x
        };

        let y = match self.h_align {
            HorizontalAlignment::_Top => self.y,
            HorizontalAlignment::_Middle => (height - self.height) / 2.0 + self.y,
            HorizontalAlignment::Bottom => (height - self.height) + self.y
        };

        (x, y)
    }

    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        let (x, y) = self.get_location(ctx);

        ctx.draw(resources.image(&self.image), x, y, self.scale);
    }

    fn clicked(&self, ctx: &mut Context, x: f32, y: f32) -> bool {
        let (pos_x, pos_y) = self.get_location(ctx);

        x >= pos_x && x <= pos_x + self.width &&
        y >= pos_y && y <= pos_y + self.height
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
        let rendered = resources.render("main", &self.text);
        //let query = rendered.query();

        /*let point = Point::new(
            self.x + rendered.width() as f32 / 2.0,
            self.y + rendered.height() as f32 / 2.0
        );*/

        ctx.draw(&rendered, self.x, self.y, 1.0);
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

    pub fn clicked(&self, ctx: &mut Context, x: f32, y: f32) -> Option<usize> {
        for (i, button) in self.buttons.iter().enumerate() {
            if button.active && button.clicked(ctx, x, y) {
                return Some(i);
            }
        }

        None
    }
}