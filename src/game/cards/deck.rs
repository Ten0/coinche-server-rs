use rand;
use rand::seq::SliceRandom;

use super::card::{Card, Suit, Value};

pub struct Deck {
	count_dealt: usize,
	// TODO: consider turning this into a Vec<Card>, for iterator
	// goodness. deck.next() producing Option<Card>?
	cards: [u8; 32],
}

#[derive(Debug)]
pub enum DeckError {
	NotEnoughCards,
}

/// translates a value between 0 and 51 to a Card. Used internally.
fn create_card_for_value(value: u8) -> Card {
	let suit = match value / 8 {
		0 => Suit::Spades,
		1 => Suit::Hearts,
		2 => Suit::Diamonds,
		3 => Suit::Clubs,
		_ => panic!("Unexpected suit conversion number"),
	};

	let value = match value % 8 {
		0 => Value::Seven,
		1 => Value::Eight,
		2 => Value::Nine,
		3 => Value::Ten,
		4 => Value::Jack,
		5 => Value::Queen,
		6 => Value::King,
		7 => Value::Ace,
		_ => panic!("Unexpected value conversion number"),
	};

	Card::new(value, suit)
}

/// A deck can be dealt from and shuffled.
impl Deck {
	/// Returns a deck where all cards are sorted by Suit, then by Value.
	pub fn new_unshuffled() -> Deck {
		let mut d = Deck {
			count_dealt: 0,
			cards: [0; 32],
		};

		let mut value = 0;
		for x in d.cards.iter_mut() {
			*x = value;
			value += 1;
		}
		d
	}

	/// A freshly shuffled deck of 32 cards.
	pub fn new_shuffled() -> Deck {
		let mut d = Deck::new_unshuffled();
		d.shuffle();
		d
	}

	/// Just pretend nothing was ever dealt.
	pub fn reset_unshuffled(&mut self) {
		self.count_dealt = 0;
	}

	/// Shuffle the cards - just as if you were starting with a fresh shuffled deck.
	pub fn reset_shuffled(&mut self) {
		self.count_dealt = 0;
		self.shuffle();
	}

	fn shuffle(&mut self) {
		self.cards.shuffle(&mut rand::thread_rng());
	}

	/// An attempt to get a card from the deck. There might not be enough.
	pub fn draw(&mut self) -> Result<Card, DeckError> {
		if self.count_dealt + 1 > 32 {
			Err(DeckError::NotEnoughCards)
		} else {
			let value = self.cards[self.count_dealt];
			self.count_dealt += 1;

			let card = create_card_for_value(value);
			Ok(card)
		}
	}

	/// An attempt to get n cards from the deck wrapped in a Vec. There might not be enough.
	pub fn draw_n(&mut self, n: usize) -> Result<Vec<Card>, DeckError> {
		if self.count_dealt + n > 32 {
			Err(DeckError::NotEnoughCards)
		} else {
			let mut cards = Vec::new();

			for _ in 0..n {
				cards.push(self.draw().ok().unwrap());
			}

			Ok(cards)
		}
	}
}
