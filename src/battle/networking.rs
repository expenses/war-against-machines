use std::thread::{spawn, sleep, JoinHandle};
use std::path::*;
use std::time::*;
use std::net::*;

use error::*;
use networking::*;
use ui::*;
use super::map::*;
use super::units::*;
use super::messages::*;
use super::paths::*;
use super::animations::*;
use super::drawer::*;

use *;
use settings::*;

// A connection from a client to a server
type ClientConn = Connection<ClientMessage, ServerMessage>;
// A connection from a server to a client
type ServerConn = Connection<ServerMessage, ClientMessage>;
// A handle to a thread that will return a result
pub type ThreadHandle = JoinHandle<Result<()>>;

pub fn client_and_server(map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<(Client, ThreadHandle)> {
	let (client_conn, server_conn) = make_connections();

	let mut server = Server::new(map, settings, server_conn)?;
	let server = spawn(move || server.run());
	let client = Client::new(client_conn)?;

	Ok((client, server))
}

pub fn client_and_multiplayer_server(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<(Client, ThreadHandle)> {
	let mut server = MultiplayerServer::new(addr, map, settings)?;
	let addr = server.addr();
	let server = spawn(move || server.run());
	let client = Client::new_from_addr(&addr.to_string())?;

	Ok((client, server))
}

// todo: it would be great to be able to use the multiplayer server for everything

struct Server {
	connection: ServerConn,
	settings: Settings,
	map: Map
}

impl Server {
	fn new(map: Either<SkirmishSettings, &Path>, settings: Settings, connection: ServerConn) -> Result<Self> {
		// Attempt to unwrap the loaded map or generate a new one based off the skirmish settings
		let mut map = match map {
			Left(settings) => Map::new_from_settings(settings),
			Right(path) => Map::load(path)?
		};

        connection.send(ServerMessage::initial_state(&mut map, Side::PlayerA))?;

        Ok(Self {
        	connection, map, settings
        })
	}

	fn run(&mut self) -> Result<()> {
		loop {
			while let Ok(message) = self.connection.recv() {
				let animations = match message {
					ClientMessage::EndTurn => {
						let mut messages = self.map.end_turn();

						// todo: AI
						if self.map.side == Side::PlayerB {
							messages = self.map.end_turn();
						}

						messages
					},
					ClientMessage::Command {unit, command} => self.map.perform_command(unit, command),
					ClientMessage::SaveGame(filename) => self.map.save(filename, &self.settings)
				};

				let player_a_animations = animations.split().0;
				self.connection.send(ServerMessage::Animations(player_a_animations))?;
			}

			sleep(Duration::from_millis(1));
		}
	}
}

pub struct Client {
	connection: ClientConn,
	pub map: Map,
	pub side: Side,
	pub animations: Vec<Animation>
}

impl Client {
	fn new(mut connection: Connection<ClientMessage, ServerMessage>) -> Result<Self> {
		let initial_state = connection.recv_blocking()?;
		let (map, side) = match initial_state {
			ServerMessage::InitialState {map, side} => (map, side),
			ServerMessage::GameFull => return Err("Game full".into()),
			message => return Err(format!("Wrong type of message recieved, expected initial state, got: {:?}", message).into())
		};

		Ok(Self {
			connection,
			map, side,
			animations: Vec::new()
		})
	}

	pub fn new_from_addr(addr: &str) -> Result<Self> {
		let client_stream = TcpStream::connect(addr)
			.chain_err(|| format!("Failed to connect to server at '{}'", addr))?;
		let connection = Connection::new_tcp(client_stream)?;
		Client::new(connection)
	}

	pub fn recv(&mut self) {
		while let Ok(message) = self.connection.recv() {
			match message {
				ServerMessage::Animations(mut animations) => self.animations.append(&mut animations),
				_ => unreachable!()
			}
		}
	}

	pub fn process_animations(&mut self, dt: f32, ctx: &mut Context, log: &mut TextDisplay, camera: &mut Camera) {
		let mut i = 0;

	    while i < self.animations.len() {
	        let status = self.animations[i].step(dt, self.side, &mut self.map, ctx, log, camera);

	        if status.finished {
	            self.animations.remove(0);
	        } else {
	            i += 1;
	        }

	        if status.blocking {
	            break;
	        }
	    }
	}

	fn send_command(&self, unit: u8, command: Command) {
		self.connection.send(ClientMessage::Command {unit, command}).unwrap();
	}

	pub fn walk(&self, unit: u8, path: &[PathPoint]) {
		self.send_command(unit, Command::walk(path));
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

	pub fn save(&self, filename: String) {
		self.connection.send(ClientMessage::SaveGame(filename)).unwrap();
	}
}

struct MultiplayerServer {
	player_a: Option<ServerConn>,
	player_b: Option<ServerConn>,
	listener: TcpListener,
	settings: Settings,
	map: Map
}

impl MultiplayerServer {
	fn addr(&self) -> SocketAddr {
		self.listener.local_addr().unwrap()
	}

	fn new(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<Self> {
		let map = match map {
			Left(settings) => Map::new_from_settings(settings),
			Right(path) => Map::load(path)?
		};

		let listener = TcpListener::bind(addr).chain_err(|| "Failed to start server")?;
		listener.set_nonblocking(true)?;
		info!("Listening for incoming connections on '{}'", listener.local_addr()?);

		Ok(Self {
			player_a: None,
			player_b: None,
			map, listener, settings
		})
	}

	fn run(&mut self) -> Result<()> {
		loop {
			// Accept new incoming connections if the players arent assigned yet
			while let Ok((stream, _)) = self.listener.accept() {
				let connection = Connection::new_tcp(stream)?;

				if self.player_a.is_none() {
					connection.send(ServerMessage::initial_state(&mut self.map, Side::PlayerA))?;
					info!("Player A connected from '{}'", connection.peer_addr()?);
					self.player_a = Some(connection);
				} else if self.player_b.is_none() {
					connection.send(ServerMessage::initial_state(&mut self.map, Side::PlayerB))?;
					info!("Player B connected from '{}'", connection.peer_addr()?);
					self.player_b = Some(connection);
				} else {
					connection.send(ServerMessage::GameFull)?;
				}
			}
			
			if self.player_a.is_some() && self.player_b.is_some() {
				let player_a = self.player_a.as_mut().unwrap();
				let player_b = self.player_b.as_mut().unwrap();

				match self.map.side {
					Side::PlayerA => {
						while let Ok(message) = player_a.recv() {
							handle_message(Side::PlayerA, &mut self.map, &player_a, &player_b, &self.settings, message);
						}

						while let Ok(_) = player_b.recv() {
							// Do nothing
						}
					},
					Side::PlayerB => {
						while let Ok(message) = player_b.recv() {
							handle_message(Side::PlayerB, &mut self.map, &player_a, &player_b, &self.settings, message);
						}

						while let Ok(_) = player_a.recv() {
							// Do nothing
						}
					}
				}
			}

			sleep(Duration::from_millis(1));
		}
	}
}

fn handle_message(side: Side, map: &mut Map, player_a: &ServerConn, player_b: &ServerConn, settings: &Settings, message: ClientMessage) {
	info!("Handling message from {}: {:?}", side, message);

	let (player_a_animations, player_b_animations) = map.handle_message(message, settings);

	if !player_a_animations.is_empty() {
		player_a.send(ServerMessage::Animations(player_a_animations)).unwrap();
	}
	if !player_b_animations.is_empty() {
		player_b.send(ServerMessage::Animations(player_b_animations)).unwrap();
	}
}