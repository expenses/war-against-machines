// A UI struct to display clickable buttons and text fields

use pedot::{self, *};
use std::ops::*;
use colours::{self, *};
use context::*;
use glutin::VirtualKeyCode;
use resources::*;

#[derive(Debug)]
pub struct ListItem {
    text: String,
    selectable: bool
}

impl ListItem {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            selectable: true
        }
    }

    pub fn unselectable(mut self) -> Self {
        self.selectable = false;
        self
    }

    pub fn set_selectable(&mut self, selectable: bool) -> &mut Self {
        self.selectable = selectable;
        self
    }

    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = text.into();
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selectable(&self) -> bool {
        self.selectable
    }

    pub fn render(&self, x: f32, y: f32, ctx: &mut Context, selected: bool) {
        let colour = if self.selectable {WHITE} else {GREY};

        if selected {
            ctx.render_text(&format!("> {}", self.text), x, y, colour);
        } else {
            ctx.render_text(&self.text, x, y, colour);
        };
    }
}

pub struct List {
    x: HorizontalAlign,
    y: VerticalAlign,
    inner: pedot::List<ListItem>,
    active: bool
}

impl List {
    pub fn new(x: f32, y: f32, items: Vec<ListItem>) -> Self {
        Self {
            x: HorizontalAlign::Middle(x),
            y: VerticalAlign::Middle(-y),
            inner: pedot::List::new(items),
            active: true
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.set_active(active);
        self
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn render(&self, ctx: &mut Context) {
        let index = self.inner.index();

        for (i, item) in self.inner.iter().enumerate() {
            let y = ctx.gui.y_absolute(self.y) + i as f32 * 20.0;
            item.render(ctx.gui.x_absolute(self.x), y, ctx, i == index && self.is_active());
        }
    }

    pub fn rotate_up(&mut self) {
        self.inner.rotate_up();
        
        while !self.get().selectable() {
            self.inner.rotate_up();
        }
    }

    pub fn rotate_down(&mut self) {
        self.inner.rotate_down();
        
        while !self.get().selectable() {
            self.inner.rotate_down();
        }
    }
}


impl Deref for List {
    type Target = pedot::List<ListItem>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Button {
    x: HorizontalAlign,
    y: VerticalAlign,
    text: &'static str
}

impl Button {
    pub fn new(x: HorizontalAlign, y: VerticalAlign, text: &'static str) -> Self {
        Self {
            x, y, text
        }
    }

    fn width(&self) -> f32 {
        Image::Button.width() * Context::UI_SCALE
    }

    fn height(&self) -> f32 {
        Image::Button.height() * Context::UI_SCALE
    }

    fn x(&self) -> HorizontalAlign {
        self.x * self.width() + self.width() / 2.0
    }

    fn y(&self) -> VerticalAlign {
        self.y * self.height() + self.height() / 2.0
    }

    fn state(&self, ctx: &Context) -> ButtonState {
        ctx.gui.button(self.x(), self.y(), self.width(), self.height())
    }

    pub fn clicked(&self, ctx: &Context) -> bool {
        self.state(ctx).is_clicked()
    }

    pub fn render(&self, ctx: &mut Context) {
        let x = ctx.gui.x_absolute(self.x());
        let y = ctx.gui.y_absolute(self.y());

        let overlay = match self.state(ctx) {
            ButtonState::None => colours::ALPHA,
            ButtonState::Hovering(_, _) => [0.0, 0.0, 0.0, 0.25],
            ButtonState::Clicked(_, _) => [1.0, 1.0, 1.0, 0.5]
        };

        ctx.render_with_overlay(Image::Button, [x, y], Context::UI_SCALE, overlay);
        ctx.render_text(self.text, x, y - Context::FONT_HEIGHT / 4.0, WHITE);
    }
}

pub struct TextInput {
    base: &'static str,
    mutable: String,
    x: HorizontalAlign,
    y: VerticalAlign
}

impl TextInput {
    pub fn new(x: HorizontalAlign, y: VerticalAlign, base: &'static str) -> Self {
        Self {
            x, y, base,
            mutable: String::new()
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        ctx.gui.key_input(&mut self.mutable, |c| c.is_alphanumeric());
        if ctx.gui.key_pressed(VirtualKeyCode::Back) {
            self.mutable.pop();
        }
    }

    pub fn render(&self, ctx: &mut Context) {
        let x = ctx.gui.x_absolute(self.x);
        let y = ctx.gui.y_absolute(self.y);
        ctx.render_text(&format!("{}{}", self.base, self.mutable), x, y, WHITE);
    }

    pub fn text(&self) -> &str {
        &self.mutable
    }
}

pub struct TextDisplay {
    text: String,
    x: HorizontalAlign,
    y: VerticalAlign
}

impl TextDisplay {
    pub fn new(x: HorizontalAlign, y: VerticalAlign) -> Self {
        Self {
            x, y, 
            text: String::new()
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn append(&mut self, line: &str) {
        self.text.push('\n');
        self.text.push_str(line);
    }

    pub fn render(&self, ctx: &mut Context) {
        let height = Context::FONT_HEIGHT * self.text.lines().count() as f32;
        let mut y = ctx.gui.y_absolute(self.y) + match self.y {
            VerticalAlign::Top(_) => Context::FONT_HEIGHT,
            VerticalAlign::Middle(_) => 0.0,
            VerticalAlign::Bottom(_) => -height
        };
        
        for line in self.text.lines() {
            let x = ctx.gui.x_absolute(self.x) + match self.x {
                HorizontalAlign::Left(_) => ctx.font_width(line) / 2.0,
                HorizontalAlign::Middle(_) => 0.0,
                HorizontalAlign::Right(_) => -ctx.font_width(line) / 2.0
            };

            ctx.render_text(line, x, y, WHITE);
            y += Context::FONT_HEIGHT;
        }
    }
}