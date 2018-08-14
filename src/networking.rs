use std::sync::mpsc::*;
use std::net::*;
use std::fmt::Debug;
use std::io::Read;

use bincode;
use serde::*;
use error::*;

pub const DEFAULT_ADDR: &str = "127.0.0.1:6666";

#[derive(Debug)]
pub enum Connection<S, R> {
	Local(Sender<S>, Receiver<R>),
	Tcp(BufferedTcp)
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
		stream.set_nonblocking(true)?;
		stream.set_nodelay(true)?;
		Ok(Connection::Tcp(BufferedTcp::new(stream)))
	}


	pub fn peer_addr(&self) -> Result<SocketAddr> {
		match *self {
			Connection::Local(_, _) => Err("Connection is over a thread, not tcp, so it doesn't not have a peer address".into()),
			Connection::Tcp(ref stream) => stream.get_inner().peer_addr().map_err(|err| err.into())
		}
	}

	pub fn recv_blocking(&mut self) -> Result<R> {
		match *self {
			Connection::Local(_, ref reciever) => reciever.recv().map_err(|err| err.to_string().into()),
			Connection::Tcp(ref mut stream) => {
				// Todo: might be wise to get the thread to sleep for a bit after failing to read a message				
				loop {
					if let Ok(message) = stream.recv() {
						return Ok(message);
					}
				}
			}
		}
	}

	pub fn recv(&mut self) -> Result<R> {
		match *self {
			Connection::Local(_, ref reciever) => reciever.try_recv().map_err(|err| err.to_string().into()),
			Connection::Tcp(ref mut stream) => stream.recv()
		}
	}

	pub fn send(&self, data: S) -> Result<()> {
		match *self {
			Connection::Local(ref sender, _) => sender.send(data).map_err(|err| err.to_string().into()),
			Connection::Tcp(ref stream) => stream.send(&data)
		}
	}
}

// todo: this struct is super useful, and the implementation wasn't super obvious, so splitting off into a seperate lib seems smart

#[derive(Debug)]
pub struct BufferedTcp {
	buffer: Vec<u8>,
	stream: TcpStream
}

impl BufferedTcp {
	fn new(stream: TcpStream) -> Self {
		Self {
			stream,
			buffer: Vec::new()
		}
	}

	fn get_inner(&self) -> &TcpStream {
		&self.stream
	}

	fn recv<R>(&mut self) -> Result<R>
		where for<'de> R: Deserialize<'de>
	{
		// Get the serialized size of a u64 (this is 8 bytes right now but could change at a later date)
		let u64_size = bincode::serialized_size(&666_u64)? as usize;

		// Append new bytes onto the buffer (but don't propagate an error if there are no new bytes)
		if self.stream.read_to_end(&mut self.buffer).is_ok() {}
		
		// Get the size of the message
		let size = bincode::deserialize::<u64>(&self.buffer)? as usize;

		// If the buffer cant contain the size and the message then return without trying to serialize
		if self.buffer.len() < u64_size + size {
			return Err("Buffer doesnt contain the struct yet".into());
		}

		// Get the message
		let message: R = bincode::deserialize(&self.buffer[u64_size .. u64_size + size])?;
		// Take the bytes out of the buffer
		self.buffer = self.buffer[u64_size + size ..].to_vec();
		Ok(message)
	}

	fn send<S: Serialize>(&self, data: &S) -> Result<()> {
		let size = bincode::serialized_size(&data)?;
		bincode::serialize_into(&self.stream, &size)?;
		bincode::serialize_into(&self.stream, &data)?;
		Ok(())
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