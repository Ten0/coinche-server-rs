use crate::prelude::*;

use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Deref, Clone)]
pub struct GameArc {
	ptr: Arc<Mutex<Game>>,
}

impl GameArc {
	pub fn new() -> Self {
		Self {
			ptr: Arc::new(Mutex::new(Game::new())),
		}
	}

	pub fn qlock(&self) -> MutexGuard<Game> {
		self.ptr.lock().unwrap()
	}
}

#[derive(Clone)]
pub struct PlayerArc {
	pub game: GameArc,
	pub player_id: usize,
}

impl PlayerArc {
	pub fn new(game: GameArc, sender: Arc<Sender>, username: String) -> crate::Result<Self> {
		let player_id = game.qlock().add_player(Player::new(sender, username))?;
		Ok(Self { game, player_id })
	}

	pub fn qlock(&self) -> PlayerPtr<MutexGuard<'_, Game>> {
		PlayerPtr {
			game: self.game.qlock(),
			player_id: self.player_id,
		}
	}
}
