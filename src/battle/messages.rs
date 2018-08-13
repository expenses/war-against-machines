use super::paths::*;
use super::map::*;
use super::units::*;
use super::animations::*;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
	EndTurn,
	Command {
		unit: u8,
		command: Command
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
	Animations(Vec<Animation>),
	InitialState {
		map: Map,
		side: Side
	}
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
	Walk(Vec<PathPoint>),
	Fire {
		x: usize,
		y: usize
	},
	Turn(UnitFacing),
	DropItem(usize),
	PickupItem(usize),
	UseItem(usize),
	ThrowItem {
		item: usize,
		x: usize,
		y: usize
	}
}