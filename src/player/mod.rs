pub mod actions;

use crate::prelude::*;

use std::ops::{Deref, DerefMut};

#[derive(Serialize)]
pub struct Player {
	pub username: String,
	#[serde(skip)]
	pub cards: Vec<Card>,
	#[serde(skip)]
	pub sender: Arc<Sender>,
}

#[derive(Clone, Copy, Deserialize)]
pub enum PlayerCardIdentifier {
	CardPos(usize),
	Card(Card),
}

impl Player {
	pub fn new(sender: Arc<Sender>, username: String) -> Self {
		Self {
			username,
			cards: Vec::new(),
			sender,
		}
	}

	pub fn send(&self, msg: impl Into<ws::Message>) -> crate::Result<()> {
		Ok(self.sender.send(msg)?)
	}

	pub fn find_card(&self, card_identifier: PlayerCardIdentifier) -> Option<(usize, Card)> {
		match card_identifier {
			PlayerCardIdentifier::CardPos(pos) => self.cards.get(pos).map(|c| (pos, *c)),
			PlayerCardIdentifier::Card(card) => self.cards.iter().position(|&c| c == card).map(|pos| (pos, card)),
		}
	}

	pub fn team(player_id: usize) -> bool {
		player_id % 2 != 0
	}
}

pub struct PlayerPtr<G: Deref<Target = Game>> {
	pub game: G,
	pub player_id: usize,
}

impl<G: Deref<Target = Game>> Deref for PlayerPtr<G> {
	type Target = Player;
	fn deref(&self) -> &Self::Target {
		&self.game.players[self.player_id]
	}
}
impl<G: Deref<Target = Game> + DerefMut<Target = Game>> DerefMut for PlayerPtr<G> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.game.players[self.player_id]
	}
}

impl<G: Deref<Target = Game>> PlayerPtr<G> {
	pub fn send_refresh_all(&self) -> crate::Result<()> {
		self.send_game_state()?;
		self.send_all_cards()?;
		Ok(())
	}

	pub fn send_game_state(&self) -> crate::Result<()> {
		self.send(ServerMessage::Game {
			game: self.game.deref(),
			player_id: self.player_id,
		})?;
		Ok(())
	}

	pub fn send_all_cards(&self) -> crate::Result<()> {
		for player in self.game.players() {
			if player.player_id != self.player_id {
				self.send(ServerMessage::CardCount {
					player_id: player.player_id,
					count: player.cards.len(),
				})?;
			}
		}
		self.send(ServerMessage::Cards {
			player_id: self.player_id,
			cards: self.cards.as_ref(),
		})?;
		Ok(())
	}

	pub fn send_player_bid(&self, player_bid: PlayerBid) -> crate::Result<()> {
		self.send(ServerMessage::PlayerBid(player_bid))
	}

	pub fn team(&self) -> bool {
		Player::team(self.player_id)
	}
}
