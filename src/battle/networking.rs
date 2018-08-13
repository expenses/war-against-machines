use std::thread::*;
use std::path::*;
use std::time::*;
use std::net::*;

use networking::*;
use super::map::*;
use super::units::*;
use super::messages::*;
use super::paths::*;
use super::animations::*;

use *;
use settings::*;

pub fn client_and_server(map: Either<SkirmishSettings, &Path>) -> Option<(Client, JoinHandle<()>)> {
	let (client_conn, server_conn) = make_connections();

	let mut server = Server::new(map, server_conn)?;
	let server = spawn(move || server.run());
	let client = Client::new(client_conn);

	Some((client, server))
}

pub fn client_and_multiplayer_server(addr: &str, map: Either<SkirmishSettings, &Path>) -> Option<(Client, JoinHandle<()>)> {
	let mut server = MultiplayerServer::new(addr, map)?;
	let addr = server.addr();
	let server = spawn(move || server.run());

	let client_stream = TcpStream::connect(addr).ok()?;
	let connection = Connection::new_tcp(client_stream);
	let client = Client::new(connection);

	Some((client, server))
}

pub fn client(addr: &str) -> Option<Client> {
	let client_stream = TcpStream::connect(addr).ok()?;
	let connection = Connection::new_tcp(client_stream);
	let client = Client::new(connection);
	Some(client)
}

struct Server {
	connection: Connection<ServerMessage, ClientMessage>,
	map: Map
}

impl Server {
	fn new(map: Either<SkirmishSettings, &Path>, connection: Connection<ServerMessage, ClientMessage>) -> Option<Self> {
		// Attempt to unwrap the loaded map or generate a new one based off the skirmish settings
		let mut map = match map {
			Left(settings) => Map::new_from_settings(settings),
			Right(path) => match Map::load(path) {
				Some(map) => map,
				_ => return None
			}
		};


        connection.send(ServerMessage::initial_state(&mut map, Side::PlayerA)).unwrap();

        Some(Self {
        	connection, map
        })
	}

	fn run(&mut self) {
		loop {
			while let Some(message) = self.connection.recv() {
				let (player_a_animations, _player_b_animations) = match message {
					ClientMessage::EndTurn => {
						let mut messages = self.map.end_turn();

						// todo: AI
						if self.map.side == Side::PlayerB {
							messages = self.map.end_turn();
						}

						messages
					},
					ClientMessage::Command {unit, command} => self.map.perform_command(unit, command)
				};

				self.connection.send(ServerMessage::Animations(player_a_animations)).unwrap();
			}

			sleep(Duration::from_millis(1));
		}
	}
}

pub struct Client {
	connection: Connection<ClientMessage, ServerMessage>,
	pub map: Map,
	pub side: Side,
	pub animations: Vec<Animation>
}

impl Client {
	fn new(mut connection: Connection<ClientMessage, ServerMessage>) -> Self {
		let (map, side) = if let Some(msg) = connection.recv_blocking() {
			if let ServerMessage::InitialState {map, side} = msg {
				(map, side)
			} else {
				panic!("{:?}", msg);
			}
		} else {
			panic!("Initial map state not recieved");
		};


		Self {
			connection,
			map, side,
			animations: Vec::new()
		}
	}

	pub fn recv(&mut self) {
		while let Some(message) = self.connection.recv() {
			match message {
				ServerMessage::Animations(mut animations) => self.animations.append(&mut animations),
				ServerMessage::InitialState {..} => unreachable!()
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

struct MultiplayerServer {
	player_a: Option<Connection<ServerMessage, ClientMessage>>,
	player_b: Option<Connection<ServerMessage, ClientMessage>>,
	listener: TcpListener,
	map: Map
}

impl MultiplayerServer {
	fn addr(&self) -> SocketAddr {
		self.listener.local_addr().unwrap()
	}

	fn new(addr: &str, map: Either<SkirmishSettings, &Path>) -> Option<Self> {
		let map = match map {
			Left(settings) => Map::new_from_settings(settings),
			Right(path) => match Map::load(path) {
				Some(map) => map,
				_ => return None
			}
		};

		let listener = TcpListener::bind(addr).unwrap();
		listener.set_nonblocking(true).unwrap();
		println!("Listening for incoming connections on '{}'", listener.local_addr().unwrap());

		Some(Self {
			player_a: None,
			player_b: None,
			map, listener
		})
	}

	fn run(&mut self) {
		loop {
			// Accept new incoming connections if the players arent assigned yet
			if self.player_a.is_none() || self.player_b.is_none() {
				while let Ok((stream, _)) = self.listener.accept() {
					let connection = Connection::new_tcp(stream);

					if self.player_a.is_none() {
						connection.send(ServerMessage::initial_state(&mut self.map, Side::PlayerA)).unwrap();
						println!("Player A connected: {:?}", connection);
						self.player_a = Some(connection);
					} else if self.player_b.is_none() {
						connection.send(ServerMessage::initial_state(&mut self.map, Side::PlayerB)).unwrap();
						println!("Player B connected: {:?}", connection);
						self.player_b = Some(connection);
					}
				}
			} else {
				let player_a = self.player_a.as_mut().unwrap();
				let player_b = self.player_b.as_mut().unwrap();

				match self.map.side {
					Side::PlayerA => {
						while let Some(message) = player_a.recv() {
							perform_command(&mut self.map, &player_a, &player_b, message);
						}

						while let Some(_) = player_b.recv() {
							// Do nothing
						}
					},
					Side::PlayerB => {
						while let Some(message) = player_b.recv() {
							perform_command(&mut self.map, &player_a, &player_b, message);
						}

						while let Some(_) = player_a.recv() {
							// Do nothing
						}
					}
				}
			}

			sleep(Duration::from_millis(1));
		}
	}
}

fn perform_command(map: &mut Map, player_a: &Connection<ServerMessage, ClientMessage>, player_b: &Connection<ServerMessage, ClientMessage>, message: ClientMessage) {
	let (player_a_animations, player_b_animations) = match message {
		ClientMessage::EndTurn => map.end_turn(),
		ClientMessage::Command {unit, command} => map.perform_command(unit, command)
	};

	if !player_a_animations.is_empty() {
		player_a.send(ServerMessage::Animations(player_a_animations)).unwrap();
	}
	if !player_b_animations.is_empty() {
		player_b.send(ServerMessage::Animations(player_b_animations)).unwrap();
	}
}