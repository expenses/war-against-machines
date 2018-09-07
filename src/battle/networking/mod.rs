use std::thread::{spawn, sleep, JoinHandle};
use std::time::*;
use std::net::*;

use error::*;
use networking::*;
use super::map::*;
use super::units::*;
use super::messages::*;
use super::paths::*;
use super::responses::*;
use super::drawer::*;
use super::ai::*;

use settings::*;

mod client;
mod server;
pub use self::client::*;
use self::server::*;

// A connection from a client to a server
pub type ClientConn = Connection<ClientMessage, ServerMessage>;
// A connection from a server to a client
pub type ServerConn = Connection<ServerMessage, ClientMessage>;
// A handle to a thread that will return a result
pub type ThreadHandle = JoinHandle<Result<()>>;

pub fn singleplayer(map: Map, settings: Settings) -> Result<(Client, ThreadHandle, ThreadHandle)> {
	let (player_conn, server_player_conn) = make_connections();
	let (ai_conn, server_ai_conn) = make_connections();

	let mut server = Server::new_local(map, server_player_conn, server_ai_conn, settings)?;
	let server = spawn(move || server.run());
	let client = Client::new(player_conn)?;
	let mut ai_client = AIClient::new(ai_conn)?;
	let ai = spawn(move || ai_client.run());

	Ok((client, ai, server))
}

pub fn multiplayer(addr: &str, map: Map, settings: Settings) -> Result<(Client, ThreadHandle)> {
	let (client_conn, server_conn) = make_connections();

	let mut server = Server::new_one_local(addr, map, server_conn, settings)?;
	let server = spawn(move || server.run());
	let client = Client::new(client_conn)?;

	Ok((client, server))
}

pub fn host_empty(addr: &str, map: Map, settings: Settings) -> Result<Server> {
	Server::new(addr, map, settings)
}

// For testing purposes
pub fn ai_vs_ai(map: Map, settings: Settings) -> Result<(Server, ThreadHandle, ThreadHandle)> {
	let (ai_1_conn, server_ai_1_conn) = make_connections();
	let (ai_2_conn, server_ai_2_conn) = make_connections();
	let server = Server::new_local(map, server_ai_1_conn, server_ai_2_conn, settings)?;

	let mut ai_1 = AIClient::new(ai_1_conn)?;
	let ai_1 = spawn(move || ai_1.run());
	let mut ai_2 = AIClient::new(ai_2_conn)?;
	let ai_2 = spawn(move || ai_2.run());

	Ok((server, ai_1, ai_2))
}