use crate::prelude::*;

#[derive(Serialize, Deserialize, Clone, Copy)]
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
	#[serde(rename = "C")]
	Capot,
}

impl BidScore {
	pub fn from_str(s: &str) -> Option<BidScore> {
		use BidScore::*;
		Some(match s {
			"80" => _80,
			"90" => _90,
			"100" => _100,
			"110" => _110,
			"120" => _120,
			"130" => _130,
			"140" => _140,
			"150" => _150,
			"160" => _160,
			"170" => _170,
			"180" => _180,
			"C" => Capot,
			_ => return None,
		})
	}

	pub fn as_str(self) -> &'static str {
		use BidScore::*;
		match self {
			_80 => "80",
			_90 => "90",
			_100 => "100",
			_110 => "110",
			_120 => "120",
			_130 => "130",
			_140 => "140",
			_150 => "150",
			_160 => "160",
			_170 => "170",
			_180 => "180",
			Capot => "C",
		}
	}

	pub fn required_points(self) -> usize {
		use BidScore::*;
		match self {
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
			Capot => 250,
		}
	}

	pub fn points(
		self,
		taking_team_points: usize,
		def_team_points: usize,
		coinche_state: CoincheState,
	) -> (usize, usize) {
		let required_points = self.required_points();
		match (taking_team_points > required_points, coinche_state) {
			(true, CoincheState::No) => (required_points, def_team_points),
			(true, CoincheState::Coinche { .. }) => (required_points * 2, 0),
			(true, CoincheState::Surcoinche { .. }) => (required_points * 4, 0),
			(false, CoincheState::No) => (0, 160 + required_points),
			(false, CoincheState::Coinche { .. }) => (0, 160 + required_points * 2),
			(false, CoincheState::Surcoinche { .. }) => (0, 160 + required_points * 4),
		}
	}
}

#[derive(Clone, Copy)]
pub struct Bid {
	pub trump: Trump,
	pub score: BidScore,
}

impl serde::Serialize for Bid {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		<String as serde::Serialize>::serialize(
			&format!("{}{}", self.score.as_str(), self.trump.as_char()),
			serializer,
		)
	}
}

impl<'de> serde::Deserialize<'de> for Bid {
	fn deserialize<D>(deserializer: D) -> Result<Bid, D::Error>
	where
		D: serde::Deserializer<'de>,
		D::Error: serde::de::Error,
	{
		use serde::de::Error;
		let mut s = <String as serde::Deserialize>::deserialize(deserializer)?;
		let trump_char = s.pop().ok_or_else(|| D::Error::custom("Empty bid value"))?;
		let trump = match trump_char {
			'T' => Trump::AllTrump,
			'A' => Trump::NoTrump,
			other => match Suit::from_char(other) {
				Some(suit) => Trump::Suit(suit),
				None => return Err(D::Error::custom("Invalid bid suit value")),
			},
		};
		let score =
			BidScore::from_str(&s).ok_or_else(|| D::Error::custom("Invalid bid score value"))?;
		Ok(Bid { score, trump })
	}
}
