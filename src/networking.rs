use std::sync::mpsc::*;
use std::net::*;
use std::fmt::Debug;

use skynet::SerializedTcpStream;
use serde::*;
use error::*;

pub const DEFAULT_ADDR: &str = "0.0.0.0:6666";

#[derive(Debug)]
pub enum Connection<S, R> {
	Local(Sender<S>, Receiver<R>),
	Tcp(SerializedTcpStream)
}

impl<S, R> Connection<S, R>
	where
		S: Serialize + Debug,
		R: Debug,
		for<'de> R: Deserialize<'de>
{
	fn new_local(sender: Sender<S>, reciever: Receiver<R>) -> Self {
		Connection::Local(sender, reciever)
	}

	pub fn new_tcp(stream: TcpStream) -> Result<Self> {
		stream.set_nodelay(true)?;
		Ok(Connection::Tcp(SerializedTcpStream::new(stream)?))
	}


	pub fn peer_addr(&self) -> Result<SocketAddr> {
		match *self {
			Connection::Local(_, _) => Err("Connection is over a thread, not tcp, so it doesn't not have a peer address".into()),
			Connection::Tcp(ref stream) => stream.inner().peer_addr().map_err(|err| err.into())
		}
	}

	pub fn recv_blocking(&mut self) -> Result<R> {
		match *self {
			Connection::Local(_, ref reciever) => reciever.recv().map_err(|err| err.to_string().into()),
			Connection::Tcp(ref mut stream) => loop {
				if let Ok(message) = stream.recv() {
					return Ok(message);
				}
			}
		}
	}

	pub fn recv(&mut self) -> Result<R> {
		match *self {
			Connection::Local(_, ref reciever) => reciever.try_recv().map_err(|err| err.to_string().into()),
			Connection::Tcp(ref mut stream) => stream.recv().map_err(|err| err.into())
		}
	}

	pub fn send(&self, data: S) -> Result<()> {
		match *self {
			Connection::Local(ref sender, _) => sender.send(data).map_err(|err| err.to_string().into()),
			Connection::Tcp(ref stream) => stream.send(&data).map_err(|err| err.into())
		}
	}
}

pub fn make_connections<S, C>() -> (Connection<C, S>, Connection<S, C>)
	where
		S: Serialize + Debug,
		C: Serialize + Debug,
		for<'de> S: Deserialize<'de>,
		for<'de> C: Deserialize<'de>,
{
	let (client_sender, server_receiver) = channel();
	let (server_sender, client_receiver) = channel();

	let client_connection = Connection::new_local(client_sender, client_receiver);
	let server_connection = Connection::new_local(server_sender, server_receiver);

	(client_connection, server_connection)
}