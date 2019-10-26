use crate::prelude::*;

use actix::prelude::*;

impl Actor for Game {
	type Context = Context<Self>;

	fn started(&mut self, _ctx: &mut Context<Self>) {
		debug!("Game actor is alive!");
	}

	fn stopped(&mut self, _ctx: &mut Context<Self>) {
		debug!("Game actor is stopped");
	}
}

pub struct ClientGameMessage {
	pub message: ClientMessage,
	pub player_id: Option<usize>,
	pub web_socket: Addr<WebSocket>,
}
impl Message for ClientGameMessage {
	type Result = Result<Option<usize>, failure::Error>;
}

impl Handler<ClientGameMessage> for Game {
	type Result = Result<Option<usize>, failure::Error>;

	fn handle(&mut self, msg: ClientGameMessage, _ctx: &mut Context<Self>) -> Self::Result {
		self.handle_msg(msg.player_id, msg.message, msg.web_socket)
	}
}
