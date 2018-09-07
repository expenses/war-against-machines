use glium::glutin::VirtualKeyCode;

use pedot::{HorizontalAlign, VerticalAlign};
use ui::*;
use context::*;
use utils::*;
use super::map::*;
use super::units::*;
use super::networking::*;
use super::responses::*;

const INVENTORY_X_OFFSET: f32 = 150.0;
const INVENTORY_UNIT_TITLE_OFFSET: f32 = -160.0;
const INVENTORY_TILE_TITLE_OFFSET: f32 = -160.0 + Context::FONT_HEIGHT;

pub enum ButtonType {
    EndTurn,
    Inventory,
    SaveGame
}

struct InventoryInfo {
    string: String,
    items: Vec<ListItem>,
    ground: Vec<ListItem>
}

impl InventoryInfo {
	fn new(unit: &Unit, map: &Map) -> Self {
		// Collect the unit's items into a vec
        let items = unit.inventory().iter()
            .map(|item| ListItem::new(&item.to_string()))
            .collect();

        // Collect the items on the ground into a vec
        let ground = map.tiles.at(unit.x, unit.y).items.iter()
            .map(|item| ListItem::new(&item.to_string()))
            .collect();
        
        Self {
            items, ground,
        	string: unit.carrying_info()
        }
	}
}

pub struct Interface {
    game_over: List,
    buttons: [Button; 3],
    save_game: TextInput,
    save_game_active: bool,
    unit_inventory: List,
    tile_inventory: List,
    unit_title: TextDisplay,
    tile_title: TextDisplay,
    inventory_active: bool,
    game_info: TextDisplay,
    log: TextDisplay
}

impl Interface {
	pub fn new() -> Self {
        Self {
            game_over: List::new(0.0, 50.0, Vec::new()).active(false),
            buttons: [
                Button::new(HorizontalAlign::Right(0.0), VerticalAlign::Bottom(0.0), "End Turn"),
                Button::new(HorizontalAlign::Right(1.0), VerticalAlign::Bottom(0.0), "Inventory"),
                Button::new(HorizontalAlign::Right(2.0), VerticalAlign::Bottom(0.0), "Save Game"),
            ],
            save_game: TextInput::new(HorizontalAlign::Middle(0.0), VerticalAlign::Middle(0.0), "Save game to: "),
            save_game_active: false,
            unit_inventory: List::new(-INVENTORY_X_OFFSET, 75.0, Vec::new()),
            tile_inventory: List::new(INVENTORY_X_OFFSET, 75.0, Vec::new()).active(false),
            unit_title: TextDisplay::new(HorizontalAlign::Middle(-INVENTORY_X_OFFSET), VerticalAlign::Middle(INVENTORY_UNIT_TITLE_OFFSET)),
            tile_title: TextDisplay::new(HorizontalAlign::Middle(INVENTORY_X_OFFSET), VerticalAlign::Middle(INVENTORY_TILE_TITLE_OFFSET)),
            inventory_active: false,
            game_info: TextDisplay::new(HorizontalAlign::Middle(10.0), VerticalAlign::Top(10.0)),
            log: TextDisplay::new(HorizontalAlign::Left(10.0), VerticalAlign::Bottom(10.0))
        }
	}

    pub fn toggle_inventory(&mut self) {
        self.inventory_active = !self.inventory_active;
    }

    pub fn toggle_save_game(&mut self) {
        self.save_game_active = !self.save_game_active;
    }

    pub fn save_game_open(&self) -> bool {
        self.save_game_active
    }

    pub fn update_savegame(&mut self, ctx: &Context, client: &Client) -> bool {
        if self.save_game_active {
            if ctx.gui.key_pressed(VirtualKeyCode::Return) {
                let filename = self.save_game.text();
                client.save(filename);
                self.save_game_active = false;
            } else {
                self.save_game.update(ctx);
            }

            true
        } else {
            false
        }
    }

    fn toggle_active_inventory(&mut self) {
        let unit = self.unit_inventory.is_active();
        self.unit_inventory.set_active(!unit);
        let tile = self.tile_inventory.is_active();
        self.tile_inventory.set_active(!tile);
    }

    fn active_inventory(&mut self) -> &mut List {
        if self.unit_inventory.is_active() {
            &mut self.unit_inventory
        } else {
            &mut self.tile_inventory
        }
    }

    pub fn try_handle_inventory_keypress(&mut self, key: VirtualKeyCode, client: &Client, selected: u8, cursor: &Option<(usize, usize)>) -> bool {
        if !self.inventory_active {
            return false;
        }
        
        match key {
            VirtualKeyCode::I => self.toggle_inventory(),
            // Rotate the selection up
            VirtualKeyCode::Up   | VirtualKeyCode::W => self.active_inventory().rotate_up(),
            // Rotate the selection down
            VirtualKeyCode::Down | VirtualKeyCode::S => self.active_inventory().rotate_down(),
            // Switch which menu is selected
            VirtualKeyCode::Left | VirtualKeyCode::Right |
            VirtualKeyCode::A    | VirtualKeyCode::D => self.toggle_active_inventory(),
            // Pick up / drop an item
            VirtualKeyCode::Return => {
                let index = self.active_inventory().index();

                if self.unit_inventory.is_active() {
                    client.drop_item(selected, index);
                } else {
                    client.pickup_item(selected, index);
                }

                //self.inventory.menu_mut(active).fit_selection();
            },
            // Use an item
            VirtualKeyCode::E => {
                if self.unit_inventory.is_active() {
                    let index = self.active_inventory().index();
                    client.use_item(selected, index);
                }
            },
            // Throw an item
            VirtualKeyCode::T => {
                if self.unit_inventory.is_active() {
                    if let Some((cursor_x, cursor_y)) = cursor {
                        client.throw_item(selected, self.active_inventory().index(), *cursor_x, *cursor_y);
                        //self.inventory.menu_mut(active).fit_selection();
                    }
                }
            },
            _ => {}
        }

        true
    }

    pub fn game_over_screen_active(&self) -> bool {
        self.game_over.is_active()
    }

    pub fn draw_game_over_screen(&self, ctx: &mut Context) {
        self.game_over.render(ctx);
    }

    pub fn clicked(&self, ctx: &Context) -> Option<ButtonType>{
        let clicked = self.buttons.iter()
            .enumerate()
            .find(|(_, button)| button.clicked(ctx))
            .map(|(i, _)| i);
        
        match clicked {
            Some(0) => Some(ButtonType::EndTurn),
            Some(1) => Some(ButtonType::Inventory),
            Some(2) => Some(ButtonType::SaveGame),
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
        self.game_info.set_text(format!("Turn {} - {}\n{}", map.turn(), side, selected));

        // Set the inventory
        if self.inventory_active {
            let info = selected_id.and_then(|selected| map.units.get(selected)).map(|unit| InventoryInfo::new(unit, map));

            // Get the name of the selected unit, it's items and the items on the ground
            if let Some(info) = info {
                self.unit_title.set_text(info.string);
                self.tile_title.set_text("Ground".into());
                
                self.unit_inventory.set_entries(vec_or_default(info.items, || ListItem::new("No items")));
                self.tile_inventory.set_entries(vec_or_default(info.ground, || ListItem::new("No items")));
            }
        }
        
        for button in &self.buttons {
            button.render(ctx);
        }

        if self.save_game_active {
            self.save_game.render(ctx);
        }

        if self.inventory_active {
            self.unit_inventory.render(ctx);
            self.tile_inventory.render(ctx);
            self.unit_title.render(ctx);
            self.tile_title.render(ctx);
        }

        self.game_info.render(ctx);
        self.log.render(ctx);
	}

    // todo: the log should probably remove items after a while
    pub fn append_to_log(&mut self, message: &str) {
        self.log.append(message);
    }

    pub fn set_game_over_screen(&mut self, stats: &GameStats) {
        self.game_over.set_entries(vec![
            ListItem::new("Game Over").unselectable(),
            ListItem::new(if stats.won {"Game Over"} else {"You Lost"}).unselectable(),
            ListItem::new(&format!("Units lost: {}", stats.units_lost)).unselectable(),
            ListItem::new(&format!("Units killed: {}", stats.units_killed)).unselectable(),
            ListItem::new("Close")
        ]);
        self.game_over.set_active(true);
        self.game_over.set_index(4);
    }
}