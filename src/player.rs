use crate::prelude::*;

use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct PlayerArc {
	pub game: GameArc,
	pub player_id: usize,
}

impl PlayerArc {
	pub fn new(
		game: GameArc,
		sender: Arc<Sender>,
		username: String,
		team: bool,
	) -> crate::Result<Self> {
		let player_id = game
			.qlock()
			.add_player(Player::new(sender, username, team))?;
		Ok(Self { game, player_id })
	}

	pub fn qlock(&self) -> PlayerPtr<MutexGuard<'_, Game>> {
		PlayerPtr {
			game: self.game.qlock(),
			player_id: self.player_id,
		}
	}
}

#[derive(Serialize)]
pub struct Player {
	pub username: String,
	pub team: bool,
	#[serde(skip)]
	pub cards: Vec<Card>,
	#[serde(skip)]
	pub sender: Arc<Sender>,
}

pub struct PlayerPtr<G: Deref<Target = Game>> {
	pub game: G,
	pub player_id: usize,
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
}

impl<G: DerefMut<Target = Game>> PlayerPtr<G> {
	pub fn bid(&mut self, bid: Option<Bid>) -> crate::Result<()> {
		let dealer_id = self.game.dealer_id;
		match self.game.game_state {
			GameState::Bidding {
				ref mut bids,
				ref coinche_state,
			} => match coinche_state {
				BiddingCoincheState::No => {
					let curr_player = bids.last().map_or(dealer_id, |b| (b.player_id + 1) % 4);
					if self.player_id == curr_player {
						let player_bid = PlayerBid {
							player_id: self.player_id,
							bid,
						};
						bids.push(player_bid);
						if bids.len() >= 4 && bids.iter().rev().take(3).all(|b| b.bid.is_none()) {
							if !self.game.try_playing_phase() {
								return Err(err_msg("Could not start game"));
							}
						} else {
							for player in self.game.players() {
								let _ = player.send_player_bid(player_bid);
							}
						}
						Ok(())
					} else {
						Err(err_msg("Not your turn"))
					}
				}
				_ => Err(err_msg("Already coinch-ed")),
			},
			_ => Err(err_msg("Not in bidding phase")),
		}
	}

	pub fn coincher(&mut self) -> crate::Result<()> {
		let team = self.team;
		let game = self.game.deref_mut();
		let players = &game.players;
		match game.game_state {
			GameState::Bidding {
				ref bids,
				ref mut coinche_state,
			} => match coinche_state {
				BiddingCoincheState::No => {
					if Some(team)
						== bids
							.last()
							.map(|last_bid| !players[last_bid.player_id].team)
					{
						*coinche_state = BiddingCoincheState::Coinche {
							player_id: self.player_id,
							player_skipped: None,
						};
						game.send_all(&ServerMessage::Coinche {
							player_id: self.player_id,
						});
						Ok(())
					} else {
						Err(err_msg("Nothing to 'coincher'"))
					}
				}
				_ => Err(err_msg("Game is in non-coinchable state")),
			},
			_ => Err(err_msg("Not in bidding phase")),
		}
	}

	pub fn surcoincher(&mut self, do_surcoinche: bool) -> crate::Result<()> {
		let team = self.team;
		let game = self.game.deref_mut();
		let players = &game.players;
		match game.game_state {
			GameState::Bidding {
				ref bids,
				ref mut coinche_state,
			} => {
				if Some(team) == bids.last().map(|last_bid| players[last_bid.player_id].team) {
					match coinche_state {
						BiddingCoincheState::Coinche {
							player_id,
							ref mut player_skipped,
						} => {
							if do_surcoinche {
								*coinche_state = BiddingCoincheState::Surcoinche {
									coincher_id: *player_id,
									surcoincher_id: self.player_id,
								};
							} else {
								if player_skipped.is_some()
									&& *player_skipped != Some(self.player_id)
								{
									return {
										if !self.game.try_playing_phase() {
											Err(err_msg("Could not start game"))
										} else {
											Ok(())
										}
									};
								} else {
									*player_skipped = Some(self.player_id);
								}
							}
							game.send_all(&ServerMessage::Coinche {
								player_id: self.player_id,
							});
							Ok(())
						}
						_ => Err(err_msg("Game is in non-sur-coinchable state")),
					}
				} else {
					Err(err_msg("Nothing to 'coincher'"))
				}
			}
			_ => Err(err_msg("Not in bidding phase")),
		}
	}
}

impl Player {
	fn new(sender: Arc<Sender>, username: String, team: bool) -> Self {
		Self {
			username,
			team,
			cards: Vec::new(),
			sender,
		}
	}

	pub fn send(&self, msg: impl Into<ws::Message>) -> crate::Result<()> {
		Ok(self.sender.send(msg)?)
	}
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
