use crate::prelude::*;

#[derive(Deserialize)]
pub enum ClientMessage {
	Init { username: String },
	RefreshGameState,
	Bid(Option<Bid>),
	Coinche,
	SurCoinche(bool),
	PlayCard(PlayerCardIdentifier),
}

#[derive(Serialize)]
pub enum ServerMessage<'a> {
	/// Player list, ids and points
	Game {
		game: &'a Game,
		player_id: usize,
	},
	/// For the connected player
	Cards {
		player_id: usize,
		cards: &'a [Card],
	},
	/// For the other players
	CardCount {
		player_id: usize,
		count: usize,
	},
	PlayerBid(PlayerBid),
	Coinche {
		player_id: usize,
	},
	SurCoinche {
		player_id: usize,
	},
	PlayedCard {
		player_id: usize,
		card_pos: usize,
		card: Card,
	},
	Trick {
		winner_id: usize,
	},
	Error {
		message: &'a str,
	},
}

impl Socket {
	pub fn handle_msg(&mut self, msg: ClientMessage) -> crate::Result<()> {
		match &self.player {
			Err(game_arc) => match msg {
				ClientMessage::Init { username } => {
					self.player = Ok(PlayerArc::new(game_arc.clone(), self.sender.clone(), username)?);
				}
				_ => return Err(err_msg("Client not initialized")),
			},
			Ok(player) => match msg {
				ClientMessage::Init { .. } => return Err(err_msg("Already initialized")),
				ClientMessage::RefreshGameState => {
					player.qlock().send_refresh_all()?;
				}
				ClientMessage::Bid(bid) => {
					player.qlock().bid(bid)?;
				}
				ClientMessage::Coinche => {
					player.qlock().coincher()?;
				}
				ClientMessage::SurCoinche(do_surcoinche) => {
					player.qlock().surcoincher(do_surcoinche)?;
				}
				ClientMessage::PlayCard(card_identifier) => {
					player.qlock().play_card(card_identifier)?;
				}
			},
		}
		Ok(())
	}
}
