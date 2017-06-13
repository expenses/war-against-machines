use std::cmp::{min, max};
use sdl2::keyboard::Keycode;

use Resources;
use context::Context;

const MIN: usize = 5;
const MAX: usize = 30;
const DEFAULT: usize = 20;

pub enum Callback {
    Play,
}

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

    fn draw(&self, ctx: &mut Context, resources: &Resources) {
        for (i, item) in self.list.iter().enumerate() {
            let mut string = String::new();

            if i == self.selection { string.push_str("> "); }
            string.push_str(item);

            let rendered = resources.render("main", string.as_str());

            let window_width = ctx.width();
            let rendered_width = rendered.query().width as f32;

            ctx.draw(&rendered, (window_width - rendered_width) / 2.0, 150.0 + i as f32 * 20.0, 1.0);
        }
    }

    fn set_item(&mut self, i: usize, string: String) {
        self.list[i] = string;
    }

    fn rotate_up(&mut self) {
        self.selection = match self.selection {
            0 => self.list.len() - 1,
            _ => self.selection - 1
        }
    }

    fn rotate_down(&mut self) {
        self.selection = (self.selection + 1) % self.list.len();
    }
}

fn to_bounds(value: usize) -> usize {
     max(min(value, MAX), MIN)
}

enum Selected {
    Main,
    Settings
}

pub struct Menu {
    pub cols: usize,
    pub rows: usize,
    main: Submenu,
    settings: Submenu,
    submenu: Selected
}

impl Menu {
    pub fn new() -> Menu {
        let cols = DEFAULT;
        let rows = DEFAULT;

        Menu {
            cols, rows,
            main: Submenu::new(vec!["Play Game".into(), "Settings".into(), "Quit".into()]),
            settings: Submenu::new(vec!["Back".into(), format!("Cols: {}", cols), format!("Rows: {}", rows)]),
            submenu: Selected::Main
        }
    }

    pub fn draw(&self, ctx: &mut Context, resources: &Resources) {
        let title = resources.image("title");

        let window_width = ctx.width();
        let title_width = title.query().width as f32;

        ctx.draw(title, (window_width - title_width) / 2.0, 0.0, 1.0);

        match self.submenu {
            Selected::Main => self.main.draw(ctx, resources),
            Selected::Settings => self.settings.draw(ctx, resources)
        }
    }

    fn refresh_settings(&mut self) {
        self.settings.set_item(1, format!("Cols: {}", self.cols));
        self.settings.set_item(2, format!("Rows: {}", self.rows));
    }

    pub fn handle_key(&mut self, ctx: &mut Context, key: Keycode) -> Option<Callback> {
        match key {
            Keycode::Up => match self.submenu {
                Selected::Main => self.main.rotate_up(),
                Selected::Settings => self.settings.rotate_up()
            },
            Keycode::Down => match self.submenu {
                Selected::Main => self.main.rotate_down(),
                Selected::Settings => self.settings.rotate_down()
            },
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
            Keycode::Left => match self.submenu {
                Selected::Settings => match self.settings.selection {
                    1 => { self.cols = to_bounds(self.cols - 5); self.refresh_settings(); },
                    2 => { self.rows = to_bounds(self.rows - 5); self.refresh_settings(); },
                    _ => {}
                },
                _ => {}
            },
            Keycode::Right => match self.submenu {
                Selected::Settings => match self.settings.selection {
                    1 => { self.cols = to_bounds(self.cols + 5); self.refresh_settings(); },
                    2 => { self.rows = to_bounds(self.rows + 5); self.refresh_settings(); },
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }

        None
    }
}