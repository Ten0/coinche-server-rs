use crate::prelude::*;

#[derive(Debug, Deserialize)]
pub enum ClientMessage {
	Init { username: String },
	RefreshGameState,
	Bid(Option<Bid>),
	Coinche,
	SurCoinche(bool),
	PlayCard(PlayerCardIdentifier),
}

#[derive(Debug, Serialize)]
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

impl Game {
	pub fn handle_msg(
		&mut self,
		player_id: Option<usize>,
		msg: ClientMessage,
		web_socket: Addr<WebSocket>,
	) -> crate::Result<Option<usize>> {
		match player_id {
			None => match msg {
				ClientMessage::Init { username } => {
					return Ok(Some(self.add_player(Player::new(username, web_socket))?));
				}
				_ => return Err(err_msg("Client not initialized")),
			},
			Some(player_id) => {
				let mut player = self.player_mut(player_id);
				match msg {
					ClientMessage::Init { .. } => return Err(err_msg("Already initialized")),
					ClientMessage::RefreshGameState => {
						player.send_refresh_all()?;
					}
					ClientMessage::Bid(bid) => {
						player.bid(bid)?;
					}
					ClientMessage::Coinche => {
						player.coincher()?;
					}
					ClientMessage::SurCoinche(do_surcoinche) => {
						player.surcoincher(do_surcoinche)?;
					}
					ClientMessage::PlayCard(card_identifier) => {
						player.play_card(card_identifier)?;
					}
				}
			}
		}
		Ok(None)
	}
}

impl<'a> ServerMessage<'a> {
	pub fn to_json_string(&self) -> String {
		serde_json::to_string(self).unwrap()
	}
}
