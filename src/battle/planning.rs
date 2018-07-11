use super::map::Map;
use super::units::*;
use super::commands::*;
use context::Context;
use ui::*;
use glutin::*;


pub trait Commander {
    fn command(&mut self, map: &Map) -> Vec<Command>;
    fn finished(&self, map: &Map) -> bool;
    fn name(&self) -> &str;
    fn draw(&mut self, ctx: &mut Context, map: &Map) {}
    fn mouse_button(&mut self, button: MouseButton, mouse: (f32, f32), ctx: &Context) {}
    fn handle_key_press(&self, key: VirtualKeyCode, pressed: bool) {}
}

struct Player {
    ui: UI,
    inventory: UI,
    selected: u8
}

impl Player {
    fn new() -> Self {
        let mut inventory = UI::new(false);

        inventory.add_text_displays(vec![
            TextDisplay::new(-75.0, 50.0, Vertical::Middle, Horizontal::Top, true),
            TextDisplay::new(75.0, 50.0, Vertical::Middle, Horizontal::Top, true)
        ]);

        inventory.add_menus(vec![
            Menu::new(-75.0, 82.5, Vertical::Middle, Horizontal::Top, true, true, Vec::new()),
            Menu::new(75.0, 62.5, Vertical::Middle, Horizontal::Top, true, false, Vec::new())
        ]);

        Self {
            inventory
        }
    }

    fn selected(&self, map: &Map) -> Option<&Unit> {
        map.units.get(self.selected)
    }

}

impl Commander for Player {
    fn command(&mut self, map: &Map) -> Vec<Command> {
        Vec::new()
    }

    fn name(&self) -> &str {
        "Player"
    }

    fn finished(&self, map: &Map) -> bool {
        false
    }

    fn mouse_button(&mut self, button: MouseButton, mouse: (f32, f32), ctx: &Context) {
        match button {
            MouseButton::Left => match self.ui.clicked(ctx, mouse) {
                // End the turn
                Some(0) => self.end_turn(),
                // Toggle the inventory
                Some(1) => self.inventory.toggle(),
                // Toggle the save game input
                Some(2) => self.ui.text_input(0).toggle(),
                // Or select/deselect a unit
                _ => if let Some((x, y)) = self.cursor {
                    self.perform_actions(x, y)
                }
            },
            MouseButton::Right => {
                if let Some((x, y)) = self.cursor {
                    if let Some(unit) = self.map.units.get_mut(self.selected) {
                        self.command_queue.push(TurnCommand::new(self.selected, UnitFacing::from_points(unit.x, unit.y, x, y)));
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, ctx: &mut Context, map: &Map) {
        // Set the inventory
        if self.inventory.active {
            // Get the name of the selected unit, it's items and the items on the ground
            let info = map.units.get(self.selected).map(|unit| {
                // Collect the unit's items into a vec
                let items: Vec<MenuItem> = unit.inventory.iter()
                    .map(|item| item!(item))
                    .collect();

                // Collect the items on the ground into a vec
                let ground: Vec<MenuItem> = map.tiles.at(unit.x, unit.y).items.iter()
                    .map(|item| item!(item))
                    .collect();

                (
                    format!(
                        "{}\n{} - {} kg\nCarry Capacity: {}/{} kg",
                        unit.name, unit.weapon, unit.weapon.tag.weight(), unit.carrying(), unit.tag.capacity()
                    ),
                    items,
                    ground
                )
            });

            // Set the inventory UI
            if let Some((unit_string, items, ground)) = info {
                self.inventory.text_display(0).text = unit_string;
                self.inventory.text_display(1).text = "Ground".into();
                self.inventory.menu(0).list = vec_or_default!(items, vec![item!("No items")]);
                self.inventory.menu(1).list = vec_or_default!(ground, vec![item!("No items")]);
            }
        }

        // Draw the UI
        self.ui.draw(ctx);
        self.inventory.draw(ctx);
    }

    fn handle_key_press(&self, key: VirtualKeyCode, pressed: bool) {
        // Respond to key presses when the inventory is open
        if self.inventory.active && pressed {
            // Get the active/inactive menu
            let (active, inactive) = if self.inventory.menu(0).selected {(0, 1)} else {(1, 0)};

            match key {
                // Toggle the inventory
                VirtualKeyCode::I => self.inventory.toggle(),
                // Rotate the selection up
                VirtualKeyCode::Up   | VirtualKeyCode::W => self.inventory.menu(active).rotate_up(),
                // Rotate the selection down
                VirtualKeyCode::Down | VirtualKeyCode::S => self.inventory.menu(active).rotate_down(),
                // Switch which menu is selected
                VirtualKeyCode::Left | VirtualKeyCode::Right |
                VirtualKeyCode::A    | VirtualKeyCode::D => {
                    self.inventory.menu(active).selected = false;
                    self.inventory.menu(inactive).selected = true;
                },
                // Pick up / drop an item
                VirtualKeyCode::Return => {
                    let index = self.inventory.menu(active).selection;

                    if let Some(unit) = self.map.units.get_mut(self.selected) {
                        // Was the item transferred?
                        let transferred = if active == 0 {
                            unit.drop_item(&mut self.map.tiles, index)
                        } else {
                            unit.pick_up_item(&mut self.map.tiles, index)
                        };

                        if transferred {
                            self.inventory.menu(active).fit_selection();
                        }
                    }
                },
                // Use an item
                VirtualKeyCode::E => {
                    if active == 0 {
                        let index = self.inventory.menu(active).selection;

                        if let Some(unit) = self.map.units.get_mut(self.selected) {
                            if unit.use_item(index) {
                                self.inventory.menu(active).fit_selection();
                            }
                        }
                    }
                },
                VirtualKeyCode::T => {
                    if let Some((id, x, y, throw, empty)) = self.selected().map(|unit| (unit.id, unit.x, unit.y, unit.tag.throw_distance(), unit.inventory.is_empty())) {
                        if !empty && active == 0 {
                            if let Some((cursor_x, cursor_y)) = self.cursor {
                                if distance_under(x, y, cursor_x, cursor_y, throw) {
                                    self.command_queue.push(ThrowItemCommand::new(id, self.inventory.menu(active).selection, cursor_x, cursor_y));
                                    self.inventory.menu(active).fit_selection();
                                }
                            }
                        }
                    }
                },
                _ => {}
            }

            return true;
        }
    }
}