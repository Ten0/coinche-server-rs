pub mod game;
pub mod player;
pub mod rcs;
pub mod server;
pub mod static_files;
pub mod websocket;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate derive_deref;

pub type Result<T> = std::result::Result<T, failure::Error>;

mod prelude {
	use super::*;
	pub use cards::*;
	pub use contract::*;
	pub use game::*;
	pub use player::*;
	pub use rcs::{GameArc, PlayerArc};
	pub use server::{ClientMessage, ServerMessage};
	pub use websocket::Socket;

	pub use failure::err_msg;
	pub use std::sync::{Arc, Mutex, MutexGuard};
	pub use ws::Sender;
}
