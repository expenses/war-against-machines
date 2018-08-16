use super::*;
use utils::*;

pub struct MultiplayerServer {
    player_a: Option<ServerConn>,
    player_b: Option<ServerConn>,
    listener: Option<TcpListener>,
    settings: Settings,
    map: Map
}

impl MultiplayerServer {
    pub fn addr(&self) -> Result<SocketAddr> {
        self.listener.as_ref().ok_or("Not running on tcp")?.local_addr().map_err(|err| err.into())
    }

    pub fn new(addr: &str, map: Either<SkirmishSettings, &Path>, settings: Settings) -> Result<Self> {
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
            listener: Some(listener),
            map, settings
        })
    }

    pub fn new_local(map: Either<SkirmishSettings, &Path>, player_a: ServerConn, player_b: ServerConn, settings: Settings) -> Result<Self> {
        let mut map = match map {
            Left(settings) => Map::new_from_settings(settings),
            Right(path) => Map::load(path)?
        };

        player_a.send(ServerMessage::initial_state(&mut map, Side::PlayerA))?;
        player_b.send(ServerMessage::initial_state(&mut map, Side::PlayerB))?;

        Ok(Self {
            player_a: Some(player_a),
            player_b: Some(player_b),
            listener: None,
            map, settings
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            // Accept new incoming connections if the players arent assigned yet
            if let Some(ref listener) = self.listener {
                while let Ok((stream, _)) = listener.accept() {
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
            }

            if self.player_a.is_some() && self.player_b.is_some() {
                let player_a = self.player_a.as_mut().unwrap();
                let player_b = self.player_b.as_mut().unwrap();

                let mut game_over = false;

                match self.map.side {
                    Side::PlayerA => {
                        while let Ok(message) = player_a.recv() {
                            game_over |= handle_message(Side::PlayerA, &mut self.map, &player_a, &player_b, &self.settings, message);
                        }

                        while let Ok(_) = player_b.recv() {
                            // Do nothing
                        }
                    },
                    Side::PlayerB => {
                        while let Ok(message) = player_b.recv() {
                            game_over |= handle_message(Side::PlayerB, &mut self.map, &player_a, &player_b, &self.settings, message);
                        }

                        while let Ok(_) = player_a.recv() {
                            // Do nothing
                        }
                    }
                }

                if game_over {
                    return Ok(())
                }
            }

            sleep(Duration::from_millis(1));
        }
    }
}

fn handle_message(side: Side, map: &mut Map, player_a: &ServerConn, player_b: &ServerConn, settings: &Settings, message: ClientMessage) -> bool {
    let mut game_over = false;
    debug!("Handling message from {}: {:?}", side, message);

    let (player_a_responses, player_b_responses) = map.handle_message(message, settings, side);

    // todo: We need to do this for ai reasons, try to fix
    let player_a_responses = vec_or_default(player_a_responses, || Response::new_state(map, Side::PlayerA));
    let player_b_responses = vec_or_default(player_b_responses, || Response::new_state(map, Side::PlayerB));

    for response in &player_a_responses {
        if let Response::GameOver(_) = response {
            game_over = true;
        }
    }

    if !player_a_responses.is_empty() {
        player_a.send(ServerMessage::Responses(player_a_responses)).unwrap();
    }
    if !player_b_responses.is_empty() {
        player_b.send(ServerMessage::Responses(player_b_responses)).unwrap();
    }

    game_over
}