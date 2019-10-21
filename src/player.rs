use crate::prelude::*;

use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};

#[derive(Serialize)]
pub struct Player {
	pub username: String,
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

	pub fn team(&self) -> bool {
		Player::team(self.player_id)
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
						let can_start_game = bids.len() >= 4 && bids.iter().rev().take(3).all(|b| b.bid.is_none());
						for player in self.game.players() {
							let _ = player.send_player_bid(player_bid);
						}
						if can_start_game {
							if !self.game.try_playing_phase() {
								return Err(err_msg("Could not start game"));
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
		let team = self.team();
		let game = self.game.deref_mut();
		match game.game_state {
			GameState::Bidding {
				ref bids,
				ref mut coinche_state,
			} => match coinche_state {
				BiddingCoincheState::No => {
					if Some(team) == bids.last().map(|last_bid| Player::team(last_bid.player_id)) {
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
		let team = self.team();
		let game = self.game.deref_mut();
		match game.game_state {
			GameState::Bidding {
				ref bids,
				ref mut coinche_state,
			} => {
				if Some(team) == bids.last().map(|last_bid| Player::team(last_bid.player_id)) {
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
								if player_skipped.is_some() && *player_skipped != Some(self.player_id) {
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

	pub fn play_card(&mut self, card_pos: usize) -> crate::Result<bool> {
		let team = self.team();
		let game = self.game.deref_mut();
		let players = &mut game.players;
		let cards = &players[self.player_id].cards;
		match game.game_state {
			GameState::Running(RunningGame {
				ref mut board,
				ref mut tricks,
				ref bid,
				..
			}) => {
				if ((board.starting_player_id + board.cards.len()) % 4) == self.player_id {
					if let Some(try_play_card) = cards.get(card_pos).copied() {
						// There's a chance we can play: it's our turn in the proper state.
						// Let's now check if the play is valid
						let can_play = if let Some(asked_suit) = board.cards.first().map(|c| c.suit) {
							if cards.iter().any(|c| c.suit == asked_suit) {
								// Forced to play the asked suit
								if try_play_card.suit != asked_suit {
									false
								} else {
									// We're the right suit. But right number?
									if bid.trump.is_trump(asked_suit) {
										// Forced to play higher if possible
										let high_trump_value = board.high_trump_value(asked_suit).unwrap();
										high_trump_value.cmp_trump(&try_play_card.value) == Ordering::Less
											|| cards
												.iter()
												.filter(|c| c.suit == asked_suit)
												.all(|c| c.value.cmp_trump(&high_trump_value) == Ordering::Less)
									} else {
										true
									}
								}
							} else {
								let should_play_trump: Option<Suit> = match bid.trump {
									Trump::Suit(trump_suit) => {
										if Player::team(board.winning_player_id(bid.trump).unwrap()) != team
											&& cards.iter().any(|c| c.suit == trump_suit)
										{
											Some(trump_suit)
										} else {
											None
										}
									}
									_ => None,
								};
								if let Some(trump_suit) = should_play_trump {
									if try_play_card.suit != trump_suit {
										false
									} else {
										// We're the right suit (trump). But right number ?
										if let Some(high_trump_value) = board.high_trump_value(trump_suit) {
											high_trump_value.cmp_trump(&try_play_card.value) == Ordering::Less
												|| cards
													.iter()
													.filter(|c| c.suit == asked_suit)
													.all(|c| c.value.cmp_trump(&high_trump_value) == Ordering::Less)
										} else {
											true
										}
									}
								} else {
									true
								}
							}
						} else {
							true
						};
						if can_play {
							board.cards.push(try_play_card);
							players[self.player_id].cards.remove(card_pos);
							for player in players.iter() {
								let _ = player.send(ServerMessage::PlayedCard {
									player_id: self.player_id,
									card: try_play_card,
									card_pos,
								});
							}
							// See if that closes the trick
							if board.cards.len() == 4 {
								let winner_id = board.winning_player_id(bid.trump).unwrap();
								tricks.push(Trick {
									winner_id,
									cards: std::mem::replace(&mut board.cards, Vec::new()),
								});
								board.starting_player_id = winner_id;
								for player in game.players() {
									let _ = player.send(ServerMessage::Trick { winner_id });
								}
							}
							game.try_end();
						}
						Ok(can_play)
					} else {
						Err(err_msg("Invalid card pos"))
					}
				} else {
					Err(err_msg("Not your turn"))
				}
			}
			_ => Err(err_msg("Games not in running state")),
		}
	}
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

	pub fn team(player_id: usize) -> bool {
		player_id % 2 != 0
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
