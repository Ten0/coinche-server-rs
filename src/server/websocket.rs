use {super::game::ClientGameMessage, crate::prelude::*};

use {
	actix::prelude::*,
	actix_web::{web, Error, HttpRequest, HttpResponse},
	actix_web_actors::ws,
	std::time::Duration,
};

/// Define http actor
#[derive(Debug)]
pub struct WebSocket {
	game_addr: Addr<Game>,
	player_id: Option<usize>,
}

impl Actor for WebSocket {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		// Keep WS alive by sending regular pings
		// (heroku's proxy disconnects idle connections)
		ctx.run_interval(Duration::from_secs(5), |_act, ctx| ctx.ping(&[]));
	}
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
	fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
		let msg = match msg {
			Ok(msg) => msg,
			Err(_) => return,
		};
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
						ctx.spawn(
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
									future::ready(())
								}),
						);
					}
				}
			}
			ws::Message::Continuation(_) => ctx.text("Not expecting continuation"),
			ws::Message::Binary(_) => ctx.text("Not expecting binary"),
			ws::Message::Close(_) => ctx.stop(),
			ws::Message::Nop | ws::Message::Pong(_) => (),
		}
	}
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JsonifiedServerMessage(pub String);
impl<'a> Handler<JsonifiedServerMessage> for WebSocket {
	type Result = ();
	fn handle(&mut self, msg: JsonifiedServerMessage, ctx: &mut Self::Context) {
		ctx.text(msg.0)
	}
}

pub async fn index(
	req: HttpRequest,
	stream: web::Payload,
	game_addr: web::Data<Addr<Game>>,
) -> Result<HttpResponse, Error> {
	ws::start(
		WebSocket {
			game_addr: game_addr.get_ref().clone(),
			player_id: None,
		},
		&req,
		stream,
	)
}
