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
	pub use cards::*;
	pub use contract::*;
	pub use game::*;
	pub use messages::{ClientMessage, ServerMessage};
	pub use player::*;
	pub use server::websocket::WebSocket;

	pub use actix::Addr;
	pub use failure::err_msg;
	pub use std::borrow::Borrow;
	pub use std::sync::{Arc, Mutex, MutexGuard};
}
