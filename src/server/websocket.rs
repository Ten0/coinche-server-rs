use super::game::ClientGameMessage;
use crate::prelude::*;

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

/// Define http actor
#[derive(Debug)]
pub struct WebSocket {
	game_addr: Addr<Game>,
	player_id: Option<usize>,
}

impl Actor for WebSocket {
	type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for WebSocket {
	fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
		match msg {
			ws::Message::Ping(msg) => ctx.pong(&msg),
			ws::Message::Text(text) => {
				debug!("Got message from {:?}: {}", self.player_id, text);
				match serde_json::from_str::<ClientMessage>(&text) {
					Err(deser_err) => ctx.text(
						ServerMessage::Error {
							message: &deser_err.to_string(),
						}
						.to_json_string(),
					),
					Ok(client_message) => {
						self.game_addr
							.send(ClientGameMessage {
								message: client_message,
								player_id: self.player_id,
								web_socket: ctx.address(),
							})
							.into_actor(self)
							.then(|res, act, ctx| {
								match res.unwrap() {
									Ok(Some(player_id)) => act.player_id = Some(player_id),
									Ok(None) => (),
									Err(err) => ctx.text(
										ServerMessage::Error {
											message: &format!("{:?}", err),
										}
										.to_json_string(),
									),
								}
								fut::ok(())
							})
							.wait(ctx);
					}
				}
			}
			ws::Message::Binary(_) => ctx.text("Not expecting binary"),
			ws::Message::Close(_) => ctx.stop(),
			ws::Message::Nop | ws::Message::Pong(_) => (),
		}
	}
}

#[derive(Message)]
pub struct JsonifiedServerMessage(pub String);
impl<'a> Handler<JsonifiedServerMessage> for WebSocket {
	type Result = ();
	fn handle(&mut self, msg: JsonifiedServerMessage, ctx: &mut Self::Context) {
		ctx.text(msg.0)
	}
}

pub fn index(req: HttpRequest, stream: web::Payload, game_addr: web::Data<Addr<Game>>) -> Result<HttpResponse, Error> {
	ws::start(
		WebSocket {
			game_addr: game_addr.get_ref().clone(),
			player_id: None,
		},
		&req,
		stream,
	)
}
