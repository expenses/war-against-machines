use super::map::*;
use super::paths::*;
use super::responses::*;
use super::units::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    EndTurn,
    SaveGame(String),
    Command { unit: u8, command: Command },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Responses(Vec<Response>),
    InitialState { map: Map, side: Side },
    GameFull,
}

impl ServerMessage {
    pub fn initial_state(map: &mut Map, side: Side) -> Self {
        ServerMessage::InitialState {
            side,
            map: map.clone_visible(side),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    Walk(Vec<UnitFacing>),
    Fire { x: usize, y: usize },
    Turn(UnitFacing),
    DropItem(usize),
    PickupItem(usize),
    UseItem(usize),
    ThrowItem { item: usize, x: usize, y: usize },
}

impl Command {
    pub fn walk(path: &[PathPoint]) -> Self {
        Command::Walk(path.iter().map(|point| point.facing).collect())
    }
}
