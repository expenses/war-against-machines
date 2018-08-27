// A UI struct to display clickable buttons and text fields

use *;

use resources::{ImageSource, Image, ToChar};
use context::Context;
use colours::{WHITE, GREY};

// The vertical alignment of an item
pub enum Vertical {
    Left,
    Middle,
    Right
}

impl Vertical {
    // Get the x position on the screen for an item of a certain width and alignment
    fn get_x(&self, mut x: f32, width: f32, ctx: &Context) -> f32 {
        x *= ctx.settings.ui_scale();

        match *self {
            Vertical::Left => x - (ctx.width - width) / 2.0,
            Vertical::Middle => x,
            Vertical::Right => x + (ctx.width - width) / 2.0
        }
    }
}

// The horizontal alignment of an item
pub enum Horizontal {
    Top,
    Middle,
    Bottom
}

impl Horizontal{
    // Get the y position on the screen for an item of a certain height and alignment
    fn get_y(&self, mut y: f32, height: f32, ctx: &Context) -> f32 {
        y *= ctx.settings.ui_scale();

        match *self {
            Horizontal::Top => (ctx.height - height) / 2.0 - y,
            Horizontal::Middle => y,
            Horizontal::Bottom => -(ctx.height - height) / 2.0 - y
        }
    }
}

// A button on the UI
pub struct Button {
    image: Image,
    x: f32,
    y: f32,
    active: bool,
    v_align: Vertical,
    h_align: Horizontal
}

impl Button {
    // Add a new button
    pub fn new(image: Image, x: f32, y: f32, v_align: Vertical, h_align: Horizontal) -> Button {     
        Button {
            x, y, v_align, h_align, image,
            active: true
        }
    }

    // Get the width of the button
    fn width(&self, ctx: &Context) -> f32 {
        self.image.width() * ctx.settings.ui_scale()
    }

    // Get the height of the button
    fn height(&self, ctx: &Context) -> f32 {
        self.image.height() * ctx.settings.ui_scale()
    }

    // Draw the button at its location and scale
    fn draw(&self, ctx: &mut Context) {
        let x = self.v_align.get_x(self.x, self.width(ctx), ctx);
        let y = self.h_align.get_y(self.y, self.height(ctx), ctx);

        let ui_scale = ctx.settings.ui_scale();
        ctx.render(self.image, [x, y], ui_scale);
    }

    // Calculate if the button was pressed
    pub fn clicked(&self, ctx: &Context, x: f32, y: f32) -> bool {
        let pos_x = self.v_align.get_x(self.x, self.width(ctx), ctx);
        let pos_y = self.h_align.get_y(self.y, self.height(ctx), ctx);

        x >= pos_x - self.width(ctx) / 2.0 && x <= pos_x + self.width(ctx) / 2.0 &&
        y >= pos_y - self.width(ctx) / 2.0 && y <= pos_y + self.height(ctx) / 2.0
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
        let mut y = self.h_align.get_y(self.y, height, ctx) + height / 2.0;
        
        for line in self.text.lines() {
            let x = self.v_align.get_x(self.x, ctx.font_width(line), ctx);
            ctx.render_text(line, x, y, WHITE);
            y -= ctx.font_height();
        }
    }
}

// A text input on the UI
pub struct TextInput {
    title: TextDisplay,
    display: String,
    text: String,
    pub active: bool
}

impl TextInput {
    // Create a new text input
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool, display: &str) -> TextInput {
        let mut text_input = TextInput {
            active,
            title: TextDisplay::new(x, y, v_align, h_align, active),
            display: display.into(),
            text: String::new()
        };

        text_input.set_text();

        text_input
    }

    fn set_text(&mut self) {
        self.title.text = [self.display.clone(), self.text.clone()].join("\n");
    }

    // Draw the text input
    fn draw(&self, ctx: &mut Context) {
        self.title.draw(ctx);
    }

    // Toggle if the text input is active or not
    pub fn toggle(&mut self) {
        self.active = !self.active;
        self.title.active = self.active;
    }

    // Handle key presses
    pub fn handle_key(&mut self, key: VirtualKeyCode) {
        if key == VirtualKeyCode::Back {
            self.text.pop();
        } else {
            self.text.push(match key.to_char() {
                '�' => return,
                character => character
            });
        }

        self.set_text();
    }

    // Get a copy of the text
    pub fn text(&self) -> String {
        self.text.clone()
    }
}

#[derive(Debug, PartialEq)]
pub struct MenuItem {
    text: String,
    enabled: bool
}

impl MenuItem {
    pub fn new(text: String, enabled: bool) -> Self {
        Self {
            text, enabled
        }
    }

    pub fn content(&self) -> String {
        self.text.clone()
    }

    // Handle key presses
    pub fn handle_key(&mut self, key: VirtualKeyCode) {
        if key == VirtualKeyCode::Back {
            self.text.pop();
        } else {
            self.text.push(match key.to_char() {
                '�' => return,
                character => character
            });
        }
    }
}

macro_rules! item {
    () => (
        MenuItem::new(String::new(), true)
    );
    ($item: expr) => (
        MenuItem::new($item.to_string(), true)
    );
    ($item: expr, $boolean: expr) => (
        MenuItem::new($item.into(), $boolean)
    );
    ($item: expr, $thing: expr, $boolean: expr) => (
        MenuItem::new(format!($item, $thing), $boolean)
    )
}

macro_rules! menu {
    ($($item: expr),*) => (
        Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, true, true, vec![$($item,)*])
    )
}

// A menu for displaying
pub struct Menu {
    pub selection: usize,
    // In a situation where there might be multiple menus, is this menu selected?
    pub selected: bool,
    list: Vec<MenuItem>,
    pub active: bool,
    x: f32,
    y: f32,
    v_align: Vertical,
    h_align: Horizontal
}

impl Menu {
    // Create a new menu
    pub fn new(x: f32, y: f32, v_align: Vertical, h_align: Horizontal, active: bool, selected: bool, list: Vec<MenuItem>) -> Menu {
        Menu {
            x, y, v_align, h_align, list, active, selected,
            selection: 0,
        }
    }

    // Draw the items in the menu
    pub fn draw(&self, ctx: &mut Context) {
        // Get the height of the rendered text
        let height = ctx.font_height() * self.list.len() as f32;
        // Get a starting y value
        let mut y = self.h_align.get_y(self.y, height, ctx) + height / 2.0;

        // Enumerate through the items
        for (i, item) in self.list.iter().enumerate() {
            let colour = if item.enabled { WHITE } else { GREY };
            
            let mut string = item.text.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if self.selected && i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let x = self.v_align.get_x(self.x, ctx.font_width(&string), ctx);
            ctx.render_text(&string, x, y, colour);
            // Decrease the y value
            y -= ctx.font_height();
        }
    }

    // Get the selected item
    pub fn selected(&self) -> String {
        self.list[self.selection].text.clone()
    }

    pub fn set_enabled(&mut self, i: usize, enabled: bool) {
        self.list[i].enabled = enabled;
    }

    pub fn enabled(&self, i: usize) -> bool {
        self.list[i].enabled
    }

    pub fn set_list(&mut self, list: Vec<MenuItem>) {
        self.list = list;
        
        if self.selection >= self.list.len() {
            self.selection = self.list.len() - 1;
        }
    }

    pub fn get_item(&self, index: usize) -> &MenuItem {
        &self.list[index]
    }

    pub fn get_item_mut(&mut self, index: usize) -> &mut MenuItem {
        &mut self.list[index]
    }

    // Are any of the items in the list enabled?
    pub fn any_enabled(&self) -> bool {
        self.list.iter().any(|item| item.enabled)
    }

    // Rotate the selection up
    pub fn rotate_up(&mut self) {
        if self.any_enabled() {
            self.selection = match self.selection {
                0 => self.list.len() - 1,
                _ => self.selection - 1
            };

            if !self.enabled(self.selection) {
                self.rotate_up();
            }
        }
    }

    // Rotate the selection down
    pub fn rotate_down(&mut self) {
        if self.any_enabled() {
            self.selection = (self.selection + 1) % self.list.len();

            if !self.enabled(self.selection) {
                self.rotate_down();
            }
        }
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

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    // Get a reference to a text display
    pub fn _text_display(&self, index: usize) -> &TextDisplay {
        &self.text_displays[index]
    }

    // Get a mutable reference to a text display
    pub fn text_display_mut(&mut self, index: usize) -> &mut TextDisplay {
        &mut self.text_displays[index]
    }

    // Get a reference to a text input
    pub fn text_input(&self, index: usize) -> &TextInput {
        &self.text_inputs[index]
    }

    // Get a mutable reference to a text input
    pub fn text_input_mut(&mut self, index: usize) -> &mut TextInput {
        &mut self.text_inputs[index]
    }

    // Get a reference to a menu
    pub fn menu(&self, index: usize) -> &Menu {
        &self.menus[index]
    }

    // Get a mutable reference to a menu
    pub fn menu_mut(&mut self, index: usize) -> &mut Menu {
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
                menu.draw(ctx);
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

#[test]
fn test_menu() {
    let items = vec![item!("Item 1"), item!("Item 2", false), item!("Item 3")];

    // Test the `item!` macro

    assert_eq!(items, vec![
        MenuItem::new("Item 1".into(), true),
        MenuItem::new("Item 2".into(), false),
        MenuItem::new("Item 3".into(), true)
    ]);

    let mut menu = Menu::new(0.0, 0.0, Vertical::Left, Horizontal::Top, true, true, items);

    assert_eq!(menu.selected(), "Item 1".to_string());

    menu.rotate_down();

    assert_eq!(menu.selected(), "Item 3".to_string());

    menu.rotate_up();

    assert_eq!(menu.selected(), "Item 1".to_string());

    menu.set_enabled(1, true);
    menu.rotate_down();

    assert_eq!(menu.selected(), "Item 2".to_string());
}