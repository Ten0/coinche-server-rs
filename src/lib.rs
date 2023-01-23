pub mod game;
pub mod logging;
pub mod messages;
pub mod player;
pub mod server;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

pub type Result<T> = std::result::Result<T, failure::Error>;

mod prelude {
	use super::*;
	pub use {
		cards::*,
		contract::*,
		game::*,
		messages::{ClientMessage, ServerMessage},
		player::*,
		server::websocket::WebSocket,
	};

	pub use {
		actix::Addr,
		failure::err_msg,
		futures::prelude::*,
		std::{
			borrow::Borrow,
			sync::{Arc, Mutex, MutexGuard},
		},
	};
}
