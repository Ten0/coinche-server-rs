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
		PlayerPtr {
			game: self,
			player_id,
		}
	}

	pub fn player_mut<'a>(&'a mut self, player_id: usize) -> PlayerPtr<&'a mut Game> {
		PlayerPtr {
			game: self,
			player_id,
		}
	}

	pub fn add_player(&mut self, player: Player) -> crate::Result<usize> {
		// Try find user again
		if let Some(id) = self
			.players
			.iter()
			.position(|p| p.username == player.username)
		{
			self.players[id] = player;
			Ok(id)
		} else {
			if self
				.players
				.iter()
				.filter(|p| p.team == player.team)
				.count() >= 2
			{
				Err(err_msg("Team is full"))
			} else {
				let id = self.players.len();
				self.players.push(player);
				if !self.try_bidding_phase() {
					let player = self.player(id);
					player.send_refresh_all()?;
					for other_player in self.players() {
						if other_player.player_id != player.player_id {
							let _ = player.send_game_state();
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
			GameState::Bidding {
				bids,
				coinche_state,
			} => {
				unimplemented!();
			}
			_ => false,
		}
	}

	pub fn try_end(&mut self) {
		if let GameState::Running(running) = &self.game_state {
			if running.tricks.iter().map(|v| v.len()).sum::<usize>() == (32 / 4) {
				let team_points = |team: bool| {
					running.tricks[team as usize]
						.iter()
						.flatten()
						.map(|c| c.points(running.bid.trump))
						.sum::<f64>()
						.floor() as usize
				};
				let taking_team_points = team_points(running.team);
				let def_team_points = team_points(!running.team);
				let (taking_points, def_points) = running.bid.score.points(
					taking_team_points,
					def_team_points,
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
	pub tricks: [Vec<Vec<Card>>; 4],
	pub coinche_state: CoincheState,
	pub board: Board,
}

#[derive(Serialize)]
pub struct Board {
	starting_player_id: usize,
	cards: Vec<Card>,
}

#[derive(Serialize, Clone, Copy)]
pub enum CoincheState {
	No,
	Coinche {
		player_id: usize,
	},
	Surcoinche {
		coincher_id: usize,
		surcoincher_id: usize,
	},
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
