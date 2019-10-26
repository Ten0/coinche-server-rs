use crate::prelude::*;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct PlayerBid {
	pub player_id: usize,
	pub bid: Option<Bid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bid {
	pub trump: Trump,
	pub score: BidScore,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum BidScore {
	#[serde(rename = "80")]
	_80,
	#[serde(rename = "90")]
	_90,
	#[serde(rename = "100")]
	_100,
	#[serde(rename = "110")]
	_110,
	#[serde(rename = "120")]
	_120,
	#[serde(rename = "130")]
	_130,
	#[serde(rename = "140")]
	_140,
	#[serde(rename = "150")]
	_150,
	#[serde(rename = "160")]
	_160,
	#[serde(rename = "170")]
	_170,
	#[serde(rename = "180")]
	_180,
	#[serde(rename = "Capot")]
	Capot,
}

impl BidScore {
	pub fn required_points(self) -> Option<usize> {
		use BidScore::*;
		Some(match self {
			_80 => 80,
			_90 => 90,
			_100 => 100,
			_110 => 110,
			_120 => 120,
			_130 => 130,
			_140 => 140,
			_150 => 150,
			_160 => 160,
			_170 => 170,
			_180 => 180,
			Capot => return None,
		})
	}

	pub fn points(
		self,
		taking_team_points: usize,
		def_team_points: usize,
		taking_team_capot: bool,
		coinche_state: CoincheState,
	) -> (usize, usize) {
		let (success, points_base) = match self.required_points() {
			Some(required_points) => (taking_team_points > required_points, required_points),
			None => (taking_team_capot, 250),
		};
		match (success, coinche_state) {
			(true, CoincheState::No) => (points_base + taking_team_points, def_team_points),
			(true, CoincheState::Coinche { .. }) => (points_base * 2 + taking_team_points, 0),
			(true, CoincheState::Surcoinche { .. }) => (points_base * 4 + taking_team_points, 0),
			(false, CoincheState::No) => (0, 160 + points_base),
			(false, CoincheState::Coinche { .. }) => (0, 160 + points_base * 2),
			(false, CoincheState::Surcoinche { .. }) => (0, 160 + points_base * 4),
		}
	}
}
