use super::*;
use ui::*;
use context::*;

pub struct Client {
    connection: ClientConn,
    pub map: Map,
    pub side: Side,
    response_queue: Vec<Response>
}

impl Client {
    pub fn new(mut connection: ClientConn) -> Result<Self> {
        let initial_state = connection.recv_blocking()?;
        let (map, side) = match initial_state {
            ServerMessage::InitialState {map, side} => (map, side),
            ServerMessage::GameFull => return Err("Game full".into()),
            message => return Err(format!("Wrong type of message recieved, expected initial state, got: {:?}", message).into())
        };

        Ok(Self {
            connection,
            map, side,
            response_queue: Vec::new()
        })
    }

    pub fn responses(&self) -> &[Response] {
        &self.response_queue
    }

    pub fn new_from_addr(addr: &str) -> Result<Self> {
        let client_stream = TcpStream::connect(addr)
            .chain_err(|| format!("Failed to connect to server at '{}'", addr))?;
        let connection = Connection::new_tcp(client_stream)?;
        Client::new(connection)
    }

    pub fn recv(&mut self) -> bool {
        let mut recieved_message = false;

        while let Ok(message) = self.connection.recv() {
            match message {
                ServerMessage::Responses(mut responses) => self.response_queue.append(&mut responses),
                _ => unreachable!()
            }

            recieved_message = true;
        }

        recieved_message
    }

    pub fn process_responses(&mut self, dt: f32, ctx: &mut Context, log: &mut TextDisplay, camera: &mut Camera) {
        let mut i = 0;

        while i < self.response_queue.len() {
            let status = self.response_queue[i].step(dt, self.side, &mut self.map, ctx, log, camera);

            if status.finished {
                self.response_queue.remove(0);
            } else {
                i += 1;
            }

            if status.blocking {
                break;
            }
        }
    }

    pub fn process_state_updates(&mut self) {
        for response in self.response_queue.drain(..) {
            if let Response::NewState(map) = response {
                self.map = map;
            }
        }
    }

    pub fn visibility_at(&self, x: usize, y: usize) -> Visibility {
        self.map.tiles.visibility_at(x, y, self.side)
    }

    pub fn our_turn(&self) -> bool {
        self.side == self.map.side
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
