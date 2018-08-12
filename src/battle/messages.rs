use super::paths::*;
use super::map::*;
use super::units::*;
use super::animations::*;

#[derive(Debug)]
pub enum ClientMessage {
	EndTurn,
	Command {
		unit: u8,
		command: Command
	}
}

#[derive(Debug)]
pub enum ServerMessage {
	Animations(Vec<Animation>),
	State(Map)
}

#[derive(Debug)]
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