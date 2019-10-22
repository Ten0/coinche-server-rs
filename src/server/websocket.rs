use crate::prelude::*;

use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// Define http actor
#[derive(Debug)]
pub struct Socket {
	state: String,
}

impl Actor for Socket {
	type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<ws::Message, ws::ProtocolError> for Socket {
	fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
		match msg {
			ws::Message::Ping(msg) => ctx.pong(&msg),
			ws::Message::Text(text) => ctx.text(std::mem::replace(&mut self.state, text)),
			ws::Message::Binary(bin) => ctx.binary(bin),
			_ => (),
		}
	}
}

pub fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
	let resp = ws::start(
		Socket {
			state: "initial state".to_owned(),
		},
		&req,
		stream,
	);
	println!("{:?}", resp);
	resp
}
