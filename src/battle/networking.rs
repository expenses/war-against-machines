use std::thread::*;
use std::path::*;
use std::time::*;

use networking::*;
use super::map::*;
use super::units::*;
use super::messages::*;
use super::paths::*;
use super::animations::*;

use settings::*;

pub fn client_and_server(settings: SkirmishSettings, map: Option<&Path>) -> Option<(Client, JoinHandle<()>)> {
	let (client_conn, server_conn) = make_connections();

	let mut server = Server::new(settings, map, server_conn)?;
	let server = spawn(move || server.run());
	let client = Client::new(client_conn);

	Some((client, server))
}

struct Server {
	connection: Connection<ServerMessage, ClientMessage>,
	map: Map
}

impl Server {
	fn new(settings: SkirmishSettings, map: Option<&Path>, connection: Connection<ServerMessage, ClientMessage>) -> Option<Self> {
		// Attempt to unwrap the loaded map or generate a new one based off the skirmish settings

		let mut map = match map {
			Some(path) => match Map::load(path) {
				Some(map) => map,
				_ => return None
			},
			None => {
				let mut map = Map::new(settings.cols, settings.rows, settings.light);

	            // Add player units
	            for x in 0 .. settings.player_units {
	                map.units.add(settings.player_unit_type, UnitSide::Player, x, 0, UnitFacing::Bottom);
	            }

	            // Add ai units
	            for y in settings.cols - settings.ai_units .. settings.cols {
	                map.units.add(settings.ai_unit_type, UnitSide::AI, y, settings.rows - 1, UnitFacing::Top);
	            }
	            
	            // Generate tiles
	            map.tiles.generate(&map.units);

	            map
			}
		};

        let client_map = map.clone_visible();

        connection.send(ServerMessage::State(client_map)).ok()?;

        Some(Self {
        	connection, map
        })
	}

	fn run(&mut self) {
		loop {
			for message in self.connection.recv_all() {
				match message {
					ClientMessage::EndTurn => {
						self.map.end_turn();

						// todo: AI
						if self.map.controller == Controller::AI {
							self.map.end_turn();
						}

						self.connection.send(ServerMessage::Animations(vec![Animation::new_state(&mut self.map)])).unwrap();
					},
					ClientMessage::Command {unit, command} => {
						let animations = self.map.perform_command(unit, command);
						self.connection.send(ServerMessage::Animations(animations)).unwrap();
					}
				}
			}

			sleep(Duration::from_millis(1));
		}
	}
}

pub struct Client {
	connection: Connection<ClientMessage, ServerMessage>,
	pub map: Map,
	pub animations: Vec<Animation>
}

impl Client {
	fn new(connection: Connection<ClientMessage, ServerMessage>) -> Self {
		let map = match connection.recv_wait() {
			Ok(msg) => match msg {
				ServerMessage::State(map) => map,
				_ => panic!("{:?}", msg)
			},
			Err(err) => panic!("{:?}", err)
		};

		Self {
			connection,
			map,
			animations: Vec::new()
		}
	}

	pub fn recv(&mut self) {
		for message in self.connection.recv_all() {
			match message {
				ServerMessage::Animations(mut animations) => {
					self.animations.append(&mut animations);
				},
				ServerMessage::State(map) => {
					self.map = map;
				}
			}
		}
	}

	fn send_command(&self, unit: u8, command: Command) {
		self.connection.send(ClientMessage::Command {unit, command}).unwrap();
	}

	pub fn walk(&self, unit: u8, path: Vec<PathPoint>) {
		self.send_command(unit, Command::Walk(path));
	}

	pub fn turn(&self, unit: u8, facing: UnitFacing) {
		self.send_command(unit, Command::Turn(facing));
	}

	pub fn fire(&self, unit: u8, x: usize, y: usize) {
		self.send_command(unit, Command::Fire {x, y});
	}

	pub fn use_item(&self, unit: u8, item: usize) {
		self.send_command(unit, Command::UseItem(item));
	}

	pub fn drop_item(&self, unit: u8, item: usize) {
		self.send_command(unit, Command::DropItem(item));
	}

	pub fn pickup_item(&self, unit: u8, item: usize) {
		self.send_command(unit, Command::PickupItem(item));
	}

	pub fn throw_item(&self, unit: u8, item: usize, x: usize, y: usize) {
		self.send_command(unit, Command::ThrowItem {item, x, y});
	}

	pub fn end_turn(&self) {
		self.connection.send(ClientMessage::EndTurn).unwrap();
	}
}