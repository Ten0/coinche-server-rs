use crate::prelude::*;

use std::sync::Arc;
use ws::{listen, CloseCode, Handler, Message, Sender};

pub struct Socket {
	pub sender: Arc<Sender>,
	pub player: Result<PlayerArc, GameArc>,
	debug: bool,
}

impl std::fmt::Display for Socket {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.player {
			Ok(player) => write!(f, "{}", &player.game.qlock().players[player.player_id].username),
			Err(_) => write!(f, "<uninitialized>"),
		}
	}
}

#[derive(Clone)]
pub struct PlayerArc {
	pub game: GameArc,
	pub player_id: usize,
}

impl PlayerArc {
	pub fn new(game: GameArc, sender: Arc<Sender>, username: String) -> crate::Result<Self> {
		let player_id = game.qlock().add_player(Player::new(sender, username))?;
		Ok(Self { game, player_id })
	}

	pub fn qlock(&self) -> PlayerPtr<MutexGuard<'_, Game>> {
		PlayerPtr {
			game: self.game.qlock(),
			player_id: self.player_id,
		}
	}
}

impl Handler for Socket {
	fn on_message(&mut self, msg: Message) -> ws::Result<()> {
		let res: crate::Result<()> = match msg.as_text() {
			Err(not_text) => Err(not_text.into()),
			Ok(text) => {
				if self.debug {
					println!("Got message from {}: {}", self, text);
				}
				match serde_json::from_str(text) {
					Ok(msg) => self.handle_msg(msg),
					Err(err) => self.sender.send(err.to_string()).map_err(|e| e.into()),
				}
			}
		};
		match res {
			Ok(()) => Ok(()),
			Err(err) => {
				let err_fmt = format!("{:?}", &err);
				println!("{}", err_fmt);
				if let Err(err_err) = self.sender.send(ServerMessage::Error { message: &err_fmt }) {
					Err(ws::Error::new(
						ws::ErrorKind::Internal,
						format!("{:?}\n\ncaused by:{}", err_err, err_fmt),
					))
				} else {
					Ok(())
				}
			}
		}
	}
	fn on_close(&mut self, _: CloseCode, _: &str) {
		self.sender.shutdown().unwrap()
	}
}

impl<'a> From<ServerMessage<'a>> for Message {
	fn from(msg: ServerMessage) -> Self {
		Message::from(&msg)
	}
}

impl<'a> From<&ServerMessage<'a>> for Message {
	fn from(msg: &ServerMessage) -> Self {
		Message::Text(serde_json::to_string(msg).expect("Could not serialize server message"))
	}
}

pub fn start_server(debug: bool) -> std::thread::JoinHandle<()> {
	let game = GameArc::new();
	let thread = std::thread::spawn(move || {
		listen("0.0.0.0:3000", |sender| Socket {
			sender: Arc::new(sender),
			player: Err(game.clone()),
			debug,
		})
		.expect("Could not start server")
	});
	// Give the server a little time to get going
	std::thread::sleep(std::time::Duration::from_millis(10));
	thread
}
