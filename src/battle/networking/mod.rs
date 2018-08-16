use std::thread::{spawn, sleep, JoinHandle};
use std::path::*;
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

use *;
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

pub fn singleplayer(map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<(Client, ThreadHandle, ThreadHandle)> {
	let (player_conn, server_player_conn) = make_connections();
	let (ai_conn, server_ai_conn) = make_connections();

	let mut server = MultiplayerServer::new_local(map, server_player_conn, server_ai_conn, settings)?;
	let server = spawn(move || server.run());
	let client = Client::new(player_conn)?;
	let mut ai_client = AIClient::new(ai_conn)?;
	let ai = spawn(move || ai_client.run());

	Ok((client, ai, server))
}

pub fn multiplayer(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<(Client, ThreadHandle)> {
	let mut server = MultiplayerServer::new(addr, map, settings)?;
	let addr = server.addr();
	let server = spawn(move || server.run());
	// todo: theres no point having the client connect via tcp as the server is running locally
	let client = Client::new_from_addr(&addr?.to_string())?;

	Ok((client, server))
}

// todo: should be able to host a multiplayer server with nobody connected yet