use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Trump {
	NoTrump,
	AllTrump,
	Suit(Suit),
}

impl Trump {
	pub fn matches(self, suit: Suit) -> TrumpMatch {
		match self {
			Self::NoTrump => TrumpMatch::NoTrump,
			Self::AllTrump => TrumpMatch::YesOrAllTrump,
			Self::Suit(trump_suit) => match trump_suit == suit {
				true => TrumpMatch::YesOrAllTrump,
				false => TrumpMatch::No,
			},
		}
	}
	pub fn as_char(self) -> char {
		match self {
			Self::NoTrump => 'A',
			Self::AllTrump => 'T',
			Self::Suit(suit) => suit.as_char(),
		}
	}
}

pub enum TrumpMatch {
	YesOrAllTrump,
	No,
	NoTrump,
}
