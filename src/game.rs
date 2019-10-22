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

#[derive(Serialize)]
pub struct Game {
	pub players: Vec<Player>,
	pub points: [usize; 2],
	pub last_round_points: [usize; 2],
	pub dealer_id: usize,
	pub game_state: GameState,
}

impl Game {
	fn new() -> Self {
		Self {
			players: Vec::new(),
			points: [0, 0],
			last_round_points: [0, 0],
			dealer_id: 0,
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
			self.players[id] = player;
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
					BiddingCoincheState::Coinche { .. } | BiddingCoincheState::Surcoinche { .. } => match bids.last() {
						Some(bid) => bid,
						None => {
							return false;
						}
					},
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
				let mut team_points: [f64; 2] = [0., 0.];
				let mut team_capot: [bool; 2] = [true, true];
				for trick in running.tricks.iter() {
					let winning_team_id = Player::team(trick.winner_id);
					team_points[winning_team_id as usize] +=
						trick.cards.iter().map(|c| c.points(running.bid.trump)).sum::<f64>();
					team_capot[!winning_team_id as usize] = false;
				}
				let taking_team_points = team_points[running.team as usize].floor() as usize;
				let def_team_points = team_points[!running.team as usize].floor() as usize;
				let taking_team_capot = team_capot[running.team as usize];
				let (taking_points, def_points) = running.bid.score.points(
					taking_team_points,
					def_team_points,
					taking_team_capot,
					running.coinche_state,
				);
				self.points[running.team as usize] += taking_points;
				self.points[!running.team as usize] += def_points;
				self.last_round_points[running.team as usize] = taking_points;
				self.last_round_points[!running.team as usize] = def_points;
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

	pub fn send_all<M: Into<ws::Message> + Copy>(&self, msg: M) {
		for player in self.players() {
			let _ = player.send(msg);
		}
	}
}

#[derive(Serialize)]
pub struct Contract {
	pub points: usize,
	pub trump: Trump,
}

#[derive(Serialize)]
pub enum GameState {
	Lobby,
	Bidding {
		bids: Vec<PlayerBid>,
		coinche_state: BiddingCoincheState,
	},
	Running(RunningGame),
}

#[derive(Serialize, Clone, Copy)]
pub struct PlayerBid {
	pub player_id: usize,
	pub bid: Option<Bid>,
}

#[derive(Serialize)]
pub struct RunningGame {
	pub team: bool,
	pub bid: Bid,
	pub tricks: Vec<Trick>,
	pub coinche_state: CoincheState,
	pub board: Board,
}

#[derive(Serialize)]
pub struct Trick {
	pub winner_id: usize,
	pub cards: Vec<Card>,
}

#[derive(Serialize)]
pub struct Board {
	pub starting_player_id: usize,
	pub cards: Vec<Card>,
}

#[derive(Serialize, Clone, Copy)]
pub enum CoincheState {
	No,
	Coinche { player_id: usize },
	Surcoinche { coincher_id: usize, surcoincher_id: usize },
}

#[derive(Serialize)]
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
					.or_else(move || asked_suit_cards.max_by(|(_i1, c1), (_i2, c2)| c1.value.cmp_trump(&c2.value))),
			}
			.expect("There is at least one card of the asked suit")
			.0 + self.starting_player_id)
				% 4,
		)
	}
}
