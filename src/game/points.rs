use crate::prelude::*;

impl Card {
	pub fn points(self, trump: Trump) -> f64 {
		use {TrumpMatch::*, Value::*};
		let base: f64 = match (trump.matches(self.suit), self.value) {
			(YesOrAllTrump, Jack) => 20.,
			(YesOrAllTrump, Nine) => 14.,
			(NoTrump, Jack) | (No, Jack) => 2.,
			(NoTrump, Nine) | (No, Nine) => 0.,
			(NoTrump, Ace) => 19.,
			(YesOrAllTrump, Ace) | (No, Ace) => 11.,
			(_, Ten) => 10.,
			(_, King) => 4.,
			(_, Queen) => 3.,
			(_, Eight) => 0.,
			(_, Seven) => 0.,
		};
		match trump {
			Trump::AllTrump => (base * 152.) / 248.,
			_ => base,
		}
	}
}
