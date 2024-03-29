use std::{cmp::Ordering, fmt};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Suit {
	Spades,
	Hearts,
	Diamonds,
	Clubs,
}

impl Suit {
	pub fn as_char(&self) -> char {
		match *self {
			Suit::Spades => 's',
			Suit::Hearts => 'h',
			Suit::Diamonds => 'd',
			Suit::Clubs => 'c',
		}
	}
	pub fn from_char(c: char) -> Option<Suit> {
		Some(match c {
			's' => Suit::Spades,
			'h' => Suit::Hearts,
			'd' => Suit::Diamonds,
			'c' => Suit::Clubs,
			_ => {
				return None;
			}
		})
	}
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Value {
	Seven,
	Eight,
	Nine,
	Jack,
	Queen,
	King,
	Ten,
	Ace,
}

impl Value {
	pub fn as_char(&self) -> char {
		match *self {
			Value::Seven => '7',
			Value::Eight => '8',
			Value::Nine => '9',
			Value::Jack => 'J',
			Value::Queen => 'Q',
			Value::King => 'K',
			Value::Ten => 'T',
			Value::Ace => 'A',
		}
	}

	pub fn cmp_trump(&self, other: &Value) -> Ordering {
		use Value::*;
		match (self, other) {
			(Jack, Jack) => Ordering::Equal,
			(Jack, _) => Ordering::Greater,
			(_, Jack) => Ordering::Less,
			(Nine, Nine) => Ordering::Equal,
			(Nine, _) => Ordering::Greater,
			(_, Nine) => Ordering::Less,
			(s, o) => std::cmp::Ord::cmp(s, o),
		}
	}
}

/// An unnamed tuple with Value and Suit.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Card {
	pub value: Value,
	pub suit: Suit,
}

impl Card {
	pub fn new(value: Value, suit: Suit) -> Card {
		Card { value, suit }
	}
}

// so cards can be printed using fmt method
impl fmt::Display for Card {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}{}", self.value.as_char(), self.suit.as_char())
	}
}
