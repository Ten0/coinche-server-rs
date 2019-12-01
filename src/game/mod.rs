pub mod cards;
pub mod contract;
pub mod points;

use crate::prelude::*;

#[derive(Debug, Serialize)]
pub struct Game {
	pub players: Vec<Player>,
	pub points: [usize; 2],
	pub round_points: Vec<RoundPoints>,
	pub dealer_id: usize,
	pub game_state: GameState,
}

#[derive(Debug, Serialize)]
pub struct RoundPoints {
	pub points: [usize; 2],
	pub bid: Bid,
	pub scored_points: [usize; 2],
	pub team: bool,
}

#[derive(Debug, Serialize)]
pub enum GameState {
	Lobby,
	Bidding {
		bids: Vec<PlayerBid>,
		coinche_state: BiddingCoincheState,
	},
	Running(RunningGame),
}

#[derive(Debug, Serialize)]
pub struct RunningGame {
	pub team: bool,
	pub bid: Bid,
	pub tricks: Vec<Trick>,
	pub coinche_state: CoincheState,
	pub board: Board,
	pub belote_player: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct Board {
	pub starting_player_id: usize,
	pub cards: Vec<Card>,
}

#[derive(Debug, Serialize)]
pub struct Trick {
	pub starting_player_id: usize,
	pub winner_id: usize,
	pub cards: Vec<Card>,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub enum CoincheState {
	No,
	Coinche { player_id: usize },
	Surcoinche { coincher_id: usize, surcoincher_id: usize },
}

#[derive(Debug, Serialize)]
pub enum BiddingCoincheState {
	No,
	Coinche {
		player_id: usize,
		player_skipped: Option<usize>,
	},
	Surcoinche {
		coincher_id: usize,
		surcoincher_id: usize,
	},
}

impl Game {
	pub fn new() -> Self {
		Self {
			players: Vec::new(),
			points: [0, 0],
			round_points: Vec::new(),
			dealer_id: 2,
			game_state: GameState::Lobby,
		}
	}

	pub fn player_ids(&self) -> std::ops::Range<usize> {
		(0..self.players.len())
	}

	pub fn players<'a>(&'a self) -> impl Iterator<Item = PlayerPtr<&'a Game>> {
		self.player_ids().map(move |i| self.player(i))
	}

	pub fn player<'a>(&'a self, player_id: usize) -> PlayerPtr<&'a Game> {
		PlayerPtr { game: self, player_id }
	}

	pub fn player_mut<'a>(&'a mut self, player_id: usize) -> PlayerPtr<&'a mut Game> {
		PlayerPtr { game: self, player_id }
	}

	pub fn add_player(&mut self, player: Player) -> crate::Result<usize> {
		// Try find user again
		if let Some(id) = self.players.iter().position(|p| p.username == player.username) {
			self.players[id].web_socket = player.web_socket;
			let player = self.player(id);
			player.send_refresh_all()?;
			Ok(id)
		} else {
			if self.players.len() >= 4 {
				Err(err_msg("Game is full"))
			} else {
				let id = self.players.len();
				self.players.push(player);
				if !self.try_bidding_phase() {
					let player = self.player(id);
					player.send_refresh_all()?;
					for other_player in self.players() {
						if other_player.player_id != player.player_id {
							let _ = other_player.send_game_state();
						}
					}
				}
				Ok(id)
			}
		}
	}

	pub fn try_bidding_phase(&mut self) -> bool {
		if self.game_state.is_lobby() && self.players.len() == 4 {
			self.dealer_id = (self.dealer_id + 1) % 4;
			let mut deck = Deck::new_shuffled();
			for player in self.players.iter_mut() {
				player.cards = deck.draw_n(32 / 4).unwrap();
			}
			self.game_state = GameState::Bidding {
				bids: Vec::new(),
				coinche_state: BiddingCoincheState::No,
			};
			self.send_refresh_all_all();
			true
		} else {
			false
		}
	}

	pub fn try_playing_phase(&mut self) -> bool {
		match &self.game_state {
			GameState::Bidding { bids, coinche_state } => {
				let player_bid: &PlayerBid = match &coinche_state {
					BiddingCoincheState::No => {
						if bids.len() >= 4 && bids.iter().rev().take(3).all(|b| b.bid.is_none()) {
							&bids[bids.len() - 4]
						} else {
							return false;
						}
					}
					BiddingCoincheState::Coinche { .. } | BiddingCoincheState::Surcoinche { .. } => bids
						.iter()
						.rev()
						.find(|b| b.bid.is_some())
						.expect("Shouldn't have C/SCed without bid"),
				};
				match player_bid.bid {
					None => {
						self.game_state = GameState::Lobby;
						self.try_bidding_phase()
					}
					Some(bid) => {
						self.game_state = GameState::Running(RunningGame {
							team: Player::team(player_bid.player_id),
							bid,
							board: Board {
								starting_player_id: (self.dealer_id + 1) % 4,
								cards: Vec::new(),
							},
							coinche_state: match *coinche_state {
								BiddingCoincheState::No => CoincheState::No,
								BiddingCoincheState::Coinche { player_id, .. } => CoincheState::Coinche { player_id },
								BiddingCoincheState::Surcoinche {
									coincher_id,
									surcoincher_id,
								} => CoincheState::Surcoinche {
									coincher_id,
									surcoincher_id,
								},
							},
							tricks: Vec::new(),
							belote_player: None,
						});
						self.send_game_state_all();
						true
					}
				}
			}
			_ => false,
		}
	}

	pub fn try_end(&mut self) {
		if let GameState::Running(running) = &self.game_state {
			if running.tricks.len() == (32 / 4) {
				let mut scored_points_f: [f64; 2] = [0., 0.];
				let mut capot: [bool; 2] = [true, true];
				for trick in running.tricks.iter() {
					let winning_team_id = Player::team(trick.winner_id);
					scored_points_f[winning_team_id as usize] +=
						trick.cards.iter().map(|c| c.points(running.bid.trump)).sum::<f64>();
					capot[!winning_team_id as usize] = false;
				}
				let mut scored_points: [usize; 2] = [0, 0];
				scored_points[0] = scored_points_f[0].floor() as usize;
				scored_points[1] = scored_points_f[1].floor() as usize;
				scored_points[Player::team(running.tricks.last().unwrap().winner_id) as usize] += 10;
				if let Some(belote_player) = running.belote_player {
					scored_points[Player::team(belote_player) as usize] += 20;
				}
				let taking_team_points = scored_points[running.team as usize];
				let def_team_points = scored_points[!running.team as usize];
				let taking_team_capot = capot[running.team as usize];
				let (taking_points, def_points) = running.bid.score.points(
					taking_team_points,
					def_team_points,
					taking_team_capot,
					running.coinche_state,
				);
				let mut round_points = [0, 0];
				round_points[running.team as usize] = taking_points;
				round_points[!running.team as usize] = def_points;
				self.points[0] += round_points[0];
				self.points[1] += round_points[1];
				self.round_points.push(RoundPoints {
					team: running.team,
					bid: running.bid,
					points: round_points,
					scored_points,
				});
				self.game_state = GameState::Lobby;
				if !self.try_bidding_phase() {
					self.send_refresh_all_all();
				}
			}
		}
	}

	pub fn send_refresh_all_all(&self) {
		for player in self.players() {
			let _ = player.send_refresh_all();
		}
	}

	pub fn send_game_state_all(&self) {
		for player in self.players() {
			let _ = player.send_game_state();
		}
	}

	pub fn send_all<'a>(&self, msg: impl Borrow<ServerMessage<'a>>) {
		for player in self.players() {
			let _ = player.send(msg.borrow());
		}
	}
}

impl GameState {
	fn is_lobby(&self) -> bool {
		match self {
			Self::Lobby => true,
			_ => false,
		}
	}
}

impl Board {
	pub fn high_trump_value(&self, asked_suit: Suit) -> Option<Value> {
		self.suit_values(asked_suit).max_by(Value::cmp_trump)
	}

	pub fn suit_values<'a>(&'a self, asked_suit: Suit) -> impl Iterator<Item = Value> + 'a {
		self.cards.iter().filter(move |c| c.suit == asked_suit).map(|c| c.value)
	}

	pub fn winning_player_id(&self, trump: Trump) -> Option<usize> {
		let asked_suit = self.cards.first()?.suit;
		let asked_suit_cards = self.cards.iter().enumerate().filter(|(_i, c)| c.suit == asked_suit);
		Some(
			(match trump {
				Trump::NoTrump => asked_suit_cards.max_by_key(|(_i, c)| c.value),
				Trump::AllTrump => asked_suit_cards.max_by(|(_i1, c1), (_i2, c2)| c1.value.cmp_trump(&c2.value)),
				Trump::Suit(trump_suit) => self
					.cards
					.iter()
					.enumerate()
					.filter(|(_i, c)| c.suit == trump_suit)
					.max_by(|(_i1, c1), (_i2, c2)| c1.value.cmp_trump(&c2.value))
					.or_else(move || asked_suit_cards.max_by_key(|(_i, c)| c.value)),
			}
			.expect("There is at least one card of the asked suit")
			.0 + self.starting_player_id)
				% 4,
		)
	}
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum BeloteRebelote {
	Belote,
	Rebelote,
}
