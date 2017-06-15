use sdl2::keyboard::Keycode;

use Resources;
use context::Context;
use utils::bound;

const MIN: usize = 10;
const MAX: usize = 50;
const DEFAULT: usize = 20;
const CHANGE: usize = 5;

// Callbacks that can be returned from key presses
pub enum Callback {
    Play,
}

// A submenu inside the main menu
struct Submenu {
    selection: usize,
    list: Vec<String>
}

impl Submenu {
    fn new(list: Vec<String>) -> Submenu {
        Submenu {
            selection: 0,
            list
        }
    }

    // Draw the items in the submenu
    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for (i, item) in self.list.iter().enumerate() {
            let mut string = item.clone();

            // If the index is the same as the selection index, push a '>' to indicate that the option is selected
            if i == self.selection { string.insert_str(0, "> "); }

            // Render the string
            let rendered = resources.render("main", &string);

            // Get the center of the rendered string
            let center = (ctx.width() - rendered.query().width) as f32 / 2.0;

            // Draw the string
            ctx.draw(&rendered, center, 150.0 + i as f32 * 20.0, 1.0);
        }
    }

    // Change an item in the list
    fn set_item(&mut self, i: usize, string: String) {
        self.list[i] = string;
    }

    // Rotate the selection up
    fn rotate_up(&mut self) {
        self.selection = match self.selection {
            0 => self.list.len() - 1,
            _ => self.selection - 1
        }
    }

    // Rotate the selection down
    fn rotate_down(&mut self) {
        self.selection = (self.selection + 1) % self.list.len();
    }
}

// Which submenu is selected
enum Selected {
    Main,
    Settings
}

// The main menu
pub struct Menu {
    pub cols: usize,
    pub rows: usize,
    main: Submenu,
    settings: Submenu,
    submenu: Selected
}

impl Menu {
    // Create a new menu
    pub fn new() -> Menu {
        Menu {
            cols: DEFAULT,
            rows: DEFAULT,
            main: Submenu::new(vec!["Play Game".into(), "Settings".into(), "Quit".into()]),
            settings: Submenu::new(vec!["Back".into(), format!("Cols: {}", DEFAULT), format!("Rows: {}", DEFAULT)]),
            submenu: Selected::Main
        }
    }

    // Draw the menu
    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        // Draw the title
        let title = resources.image(&"title".into());
        let center = (ctx.width() - title.query().width) as f32 / 2.0;
        ctx.draw(title, center, 0.0, 1.0);

        // Draw the selected submenu
        match self.submenu {
            Selected::Main => self.main.draw(ctx, resources),
            Selected::Settings => self.settings.draw(ctx, resources)
        }
    }

    // Refresh the settings submenu
    fn refresh_settings(&mut self) {
        self.settings.set_item(1, format!("Cols: {}", self.cols));
        self.settings.set_item(2, format!("Rows: {}", self.rows));
    }

    // Handle key presses
    pub fn handle_key(&mut self, ctx: &mut Context, key: Keycode) -> Option<Callback> {
        match key {
            // Rotate the selections up
            Keycode::Up => match self.submenu {
                Selected::Main => self.main.rotate_up(),
                Selected::Settings => self.settings.rotate_up()
            },
            // Rotate the selections down
            Keycode::Down => match self.submenu {
                Selected::Main => self.main.rotate_down(),
                Selected::Settings => self.settings.rotate_down()
            },
            // Perform actions on the selection 
            Keycode::Return => match self.submenu {
                Selected::Main => match self.main.selection {
                    0 => return Some(Callback::Play),
                    1 => self.submenu = Selected::Settings,
                    2 => ctx.quit(),
                    _ => {}
                },
                Selected::Settings => match self.settings.selection {
                    0 => self.submenu = Selected::Main,
                    _ => {}
                }
            },
            // Lower the cols/rows values
            Keycode::Left => match self.submenu {
                Selected::Settings => match self.settings.selection {
                    1 => { self.cols = bound(self.cols - CHANGE, MIN, MAX); self.refresh_settings(); },
                    2 => { self.rows = bound(self.rows - CHANGE, MIN, MAX); self.refresh_settings(); },
                    _ => {}
                },
                _ => {}
            },
            // Raise the cols/rows values
            Keycode::Right => match self.submenu {
                Selected::Settings => match self.settings.selection {
                    1 => { self.cols = bound(self.cols + CHANGE, MIN, MAX); self.refresh_settings(); },
                    2 => { self.rows = bound(self.rows + CHANGE, MIN, MAX); self.refresh_settings(); },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        None
    }
}