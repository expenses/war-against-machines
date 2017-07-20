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

// Get the x position on the screen for an item of a certain width and alignment
fn get_x(x: f32, width: f32, screen_width: f32, v_align: &Vertical) -> f32 {
    match *v_align {
        Vertical::Left => x - (screen_width - width) / 2.0,
        Vertical::Middle => x,
        Vertical::Right => x + (screen_width - width) / 2.0
    }
}

// Get the y position on the screen for an item of a certain height and alignment
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
            let x = get_x(self.x, ctx.font_width(line), ctx.width, &self.v_align);
            ctx.render_text(line, x, y, WHITE);
            y -= ctx.font_height();
        }
    }
}

// A text input on the UI
pub struct TextInput {
    title: TextDisplay,
    input: TextDisplay,
    pub active: bool
}

impl TextInput {
    // Create a new text input
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool, ctx: &Context, display: &str) -> TextInput {
        let mut title = TextDisplay::new(x, y, v_align, h_align, active);
        title.text = display.into();

        TextInput {
            title, active,
            input: TextDisplay::new(x, y - ctx.font_height(), v_align, h_align, active)
        }
    }

    // Draw the text input
    fn draw(&self, ctx: &mut Context) {
        self.title.draw(ctx);
        self.input.draw(ctx);
    }

    // Toggle if the text input is active or not
    pub fn toggle(&mut self) {
        self.active = !self.active;
        self.title.active = self.active;
        self.input.active = self.active;
    }

    // Handle key presses
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

    // Get a copy of the text
    pub fn text(&self) -> String {
        self.input.text.clone()
    }
}

// A menu for displaying
pub struct Menu {
    pub selection: usize,
    // In a situation where there might be multiple menus, is this menu selected?
    pub selected: bool,
    pub list: Vec<String>,
    pub active: bool,
    x: f32,
    y: f32,
    v_align: Vertical,
    h_align: Horizontal
}

impl Menu {
    // Create a new menu
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool, selected: bool, list: Vec<String>) -> Menu {
        Menu {
            x, y, v_align, h_align, list, active, selected,
            selection: 0,
        }
    }

    // Draw the items in the menu
    pub fn render(&self, ctx: &mut Context) {
        // Get the height of the rendered text
        let height = ctx.font_height() * self.list.len() as f32;
        // Get a starting y value
        let mut y = get_y(self.y, height, ctx.height, &self.h_align) + height / 2.0;

        // Enumerate through the items
        for (i, item) in self.list.iter().enumerate() {
            let mut string = item.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if self.selected && i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let x = get_x(self.x, ctx.font_width(&string), ctx.width, &self.v_align);
            ctx.render_text(&string, x, y, WHITE);
            // Decrease the y value
            y -= ctx.font_height();
        }
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    pub fn push(&mut self, item: String) {
        self.list.push(item);
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    // Get the selected item
    pub fn selected(&self) -> String {
        self.list[self.selection].clone()
    }

    // Change an item in the list
    pub fn set_item(&mut self, i: usize, string: String) {
        self.list[i] = string;
    }

    // Rotate the selection up
    pub fn rotate_up(&mut self) {
        self.selection = match self.selection {
            0 => self.list.len() - 1,
            _ => self.selection - 1
        }
    }

    // Rotate the selection down
    pub fn rotate_down(&mut self) {
        self.selection = (self.selection + 1) % self.list.len();
    }
}

// The UI struct
pub struct UI {
    pub active: bool,
    buttons: Vec<Button>,
    text_displays: Vec<TextDisplay>,
    text_inputs: Vec<TextInput>,
    menus: Vec<Menu>
}

impl UI {
    // Create a new UI with vecs of buttons and text displays
    pub fn new(active: bool) -> UI {
        UI {
            active,
            buttons: Vec::new(),
            text_displays: Vec::new(),
            text_inputs: Vec::new(),
            menus: Vec::new()
        }
    }

    pub fn add_buttons(&mut self, mut buttons: Vec<Button>) {
        self.buttons.append(&mut buttons);
    }

    pub fn add_text_displays(&mut self, mut text_displays: Vec<TextDisplay>) {
        self.text_displays.append(&mut text_displays);
    }

    pub fn add_text_inputs(&mut self, mut text_inputs: Vec<TextInput>) {
        self.text_inputs.append(&mut text_inputs);
    }

    pub fn add_menus(&mut self, mut menus: Vec<Menu>) {
        self.menus.append(&mut menus);
    }

    // Get a mutable reference to a text display
    pub fn text_display(&mut self, index: usize) -> &mut TextDisplay {
        &mut self.text_displays[index]
    }

    // Get a mutable reference to a text input
    pub fn text_input(&mut self, index: usize) -> &mut TextInput {
        &mut self.text_inputs[index]
    }

    // Get a mutable reference to a menu
    pub fn menu(&mut self, index: usize) -> &mut Menu {
        &mut self.menus[index]
    }

    // Draw all the active buttons, text displays and text inputs
    pub fn draw(&self, ctx: &mut Context) {
        if !self.active {
            return;
        }

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

        for menu in &self.menus {
            if menu.active {
                menu.render(ctx);
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