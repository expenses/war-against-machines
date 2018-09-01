use glium::glutin::VirtualKeyCode;

use pedot::*;

use ui;
use ui::*;
use resources::*;
use context::*;
use utils::*;
use super::map::*;
use super::units::*;
use super::networking::*;
use super::responses::*;

pub enum Button {
    EndTurn,
    Inventory,
    SaveGame
}

struct InventoryInfo {
    string: String,
    items: Vec<MenuItem>,
    ground: Vec<MenuItem>
}

impl InventoryInfo {
	fn new(unit: &Unit, map: &Map) -> Self {
		// Collect the unit's items into a vec
        let items = unit.inventory().iter()
            .map(|item| item!(item))
            .collect();

        // Collect the items on the ground into a vec
        let ground = map.tiles.at(unit.x, unit.y).items.iter()
            .map(|item| item!(item))
            .collect();
        
        Self {
            items, ground,
        	string: unit.carrying_info()
        }
	}
}

pub struct Interface {
    general: UI,
    inventory: UI,
    game_over: List<ListItem>,
    game_over_active: bool
}

impl Interface {
	pub fn new() -> Self {
		let width_offset = -Image::EndTurnButton.width();

        let mut general = UI::new(true);

        // Buttons
        general.add_buttons(vec![
            ui::Button::new(Image::EndTurnButton, 0.0, 0.0, Vertical::Right, Horizontal::Bottom),
            ui::Button::new(Image::InventoryButton, width_offset, 0.0, Vertical::Right, Horizontal::Bottom),
            ui::Button::new(Image::SaveGameButton, width_offset * 2.0, 0.0, Vertical::Right, Horizontal::Bottom)
        ]);

        general.add_text_displays(vec![
        	// Game Info
            TextDisplay::new(0.0, 10.0, Vertical::Middle, Horizontal::Top, true),
            // Game log
            TextDisplay::new(10.0, -10.0, Vertical::Left, Horizontal::Bottom, true)
        ]);

        // Save game
        general.add_text_inputs(vec![
            TextInput::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, "Save game to:")
        ]);

        // Game over screen
        general.add_menus(vec![
            Menu::new(0.0, 0.0, Vertical::Middle, Horizontal::Middle, false, true, Vec::new())
        ]);

        // Create the inventory UI

        let mut inventory = UI::new(false);
        
        inventory.add_text_displays(vec![
            // Unit title
            TextDisplay::new(-75.0, 50.0, Vertical::Middle, Horizontal::Top, true),
            // Ground title
            TextDisplay::new(75.0, 50.0, Vertical::Middle, Horizontal::Top, true)
        ]);

        inventory.add_menus(vec![
            // Unit inventory
            Menu::new(-75.0, 82.5, Vertical::Middle, Horizontal::Top, true, true, Vec::new()),
            // Ground inventory
            Menu::new(75.0, 62.5, Vertical::Middle, Horizontal::Top, true, false, Vec::new())
        ]);

        Self {
        	general, inventory,
            game_over: list!(),
            game_over_active: false
        }
	}

    pub fn toggle_inventory(&mut self) {
        self.inventory.toggle();
    }

    pub fn toggle_save_game(&mut self) {
        self.general.text_input_mut(0).toggle()
    }

    pub fn try_handle_save_game_keypress(&mut self, key: VirtualKeyCode, client: &Client) -> bool {
        if self.general.text_input(0).active {
            if key == VirtualKeyCode::Return {
                let filename = self.general.text_input(0).text();
                client.save(filename);
                self.general.text_input_mut(0).toggle();
            } else {
                self.general.text_input_mut(0).handle_key(key);
            }

            true
        } else {
            false
        }
    }

    pub fn try_handle_inventory_keypress(&mut self, key: VirtualKeyCode, client: &Client, selected: u8, cursor: &Option<(usize, usize)>) -> bool {
        if !self.inventory.active {
            return false;
        }

        // Get the active/inactive menu
        let (active, inactive) = if self.inventory.menu(0).selected {(0, 1)} else {(1, 0)};
        
        match key {
            // Toggle the inventory
            VirtualKeyCode::I => self.inventory.toggle(),
            // Rotate the selection up
            VirtualKeyCode::Up   | VirtualKeyCode::W => self.inventory.menu_mut(active).rotate_up(),
            // Rotate the selection down
            VirtualKeyCode::Down | VirtualKeyCode::S => self.inventory.menu_mut(active).rotate_down(),
            // Switch which menu is selected
            VirtualKeyCode::Left | VirtualKeyCode::Right |
            VirtualKeyCode::A    | VirtualKeyCode::D => {
                self.inventory.menu_mut(active).selected = false;
                self.inventory.menu_mut(inactive).selected = true;
            },
            // Pick up / drop an item
            VirtualKeyCode::Return => {
                let index = self.inventory.menu(active).selection;

                if active == 0 {
                    client.drop_item(selected, index);
                } else {
                    client.pickup_item(selected, index);
                }

                //self.inventory.menu_mut(active).fit_selection();
            },
            // Use an item
            VirtualKeyCode::E => {
                if active == 0 {
                    let index = self.inventory.menu(active).selection;
                    client.use_item(selected, index);
                }
            },
            // Throw an item
            VirtualKeyCode::T => {
                if active == 0 {
                    if let Some((cursor_x, cursor_y)) = cursor {
                        client.throw_item(selected, self.inventory.menu(active).selection, *cursor_x, *cursor_y);
                        //self.inventory.menu_mut(active).fit_selection();
                    }
                }
            },
            _ => {}
        }

        true
    }

    pub fn game_over_screen_active(&self) -> bool {
        self.game_over_active
    }

    pub fn draw_game_over_screen(&self, ctx: &mut Context) {
        render_list(&self.game_over, ctx);
    }

    pub fn clicked(&self, ctx: &Context, mouse: (f32, f32)) -> Option<Button>{
        match self.general.clicked(ctx, mouse) {
            Some(0) => Some(Button::EndTurn),
            Some(1) => Some(Button::Inventory),
            Some(2) => Some(Button::SaveGame),
            _ => None
        }
    }

	pub fn draw(&mut self, ctx: &mut Context, selected_id: Option<u8>, map: &Map, verses_ai: bool) {
		// Get a string of info about the selected unit
        let selected = selected_id
            .and_then(|selected| map.units.get(selected))
            .map(|unit| unit.info())
            .unwrap_or_else(String::new);

        let side = if verses_ai {
            map.side.vs_ai_string()
        } else {
            map.side.multiplayer_string()
        };

        // Set the text of the UI text display
        self.general.text_display_mut(0).text = format!("Turn {} - {}\n{}", map.turn(), side, selected);

        // Set the inventory
        if self.inventory.active {
            let info = selected_id.and_then(|selected| map.units.get(selected)).map(|unit| InventoryInfo::new(unit, map));

            // Get the name of the selected unit, it's items and the items on the ground
            if let Some(info) = info {
                self.inventory.text_display_mut(0).text = info.string;
                self.inventory.text_display_mut(1).text = "Ground".into();
                self.inventory.menu_mut(0).set_list(vec_or_default(info.items, || item!("No items")));
                self.inventory.menu_mut(1).set_list(vec_or_default(info.ground, || item!("No items")));
            }
        }
        
        // Draw the UI
        self.general.draw(ctx);
        self.inventory.draw(ctx);
	}

    // todo: the log should probably remove items after a while
    pub fn append_to_log(&mut self, message: &str) {
        self.general.text_display_mut(1).append(message)
    }

    pub fn set_game_over_screen(&mut self, stats: &GameStats) {
        self.game_over_active = true;
        self.game_over.set_entries(vec![
            ListItem::new(0.0, 40.0, "Game Over").unselectable(),
            ListItem::new(0.0, 20.0, if stats.won {"Game Over"} else {"You Lost"}).unselectable(),
            ListItem::new(0.0, 0.0, &format!("Units lost: {}", stats.units_lost)).unselectable(),
            ListItem::new(0.0, -20.0, &format!("Units killed: {}", stats.units_killed)).unselectable(),
            ListItem::new(0.0, -40.0, "Close")
        ]);
        self.game_over.set_index(4);
    }
}