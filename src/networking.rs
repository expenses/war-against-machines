use std::sync::mpsc::*;

pub struct Connection<S, R> {
	sender: Sender<S>,
	reciever: Receiver<R>
}

impl<S, R> Connection<S, R> {
	fn new(sender: Sender<S>, reciever: Receiver<R>) -> Self {
		Self {
			sender, reciever
		}
	}

	pub fn recv_wait(&self) -> Result<R, RecvError> {
		self.reciever.recv()
	}

	pub fn recv_all(&self) -> TryIter<R> {
		self.reciever.try_iter()
	}

	pub fn send(&self, data: S) -> Result<(), SendError<S>> {
		self.sender.send(data)
	}
}

pub fn make_connections<S, C>() -> (Connection<C, S>, Connection<S, C>) {
	let (client_sender, server_receiver) = channel();
	let (server_sender, client_receiver) = channel();

	let client_connection = Connection::new(client_sender, client_receiver);
	let server_connection = Connection::new(server_sender, server_receiver);

	(client_connection, server_connection)
}