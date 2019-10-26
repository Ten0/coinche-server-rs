pub mod game;
pub mod websocket;

use crate::prelude::*;

use actix::prelude::*;
use actix_files as fs;
use actix_service::Service;
use actix_web::{dev::Server, http::header, middleware, web, App, HttpResponse, HttpServer};
use futures::future::{ok, Either};
use std::{sync::mpsc, thread};

pub fn start(port: u16) -> Server {
	let (tx, rx) = mpsc::channel();

	thread::spawn(move || {
		let sys = actix_rt::System::new("http-server");

		let game_addr = Game::new().start();

		let webserver_addr = HttpServer::new(move || {
			App::new()
				.data(game_addr.clone())
				.wrap(middleware::Logger::default()) // Enable logger
				.wrap(middleware::Compress::default()) // Enable compression if client asks for it
				.wrap_fn(|req, srv| {
					// Enforce HTTPS if forwarded from http (heroku)
					let headers = req.headers();
					if headers.get("X-Forwarded-Proto").map_or(false, |v| v == "http") {
						let host_header = headers.get(header::HOST);
						let host_header_str = host_header.and_then(|h| h.to_str().ok()).unwrap_or("perdu.com");
						let location = format!("https://{}{}", host_header_str, req.path());
						Either::B(ok(req.into_response(HttpResponse::Found().header(header::LOCATION, location).finish().into_body())))
					} else {
						Either::A(srv.call(req))
					}
				})
				.route("/ws/", web::get().to(websocket::index)) // websocket route
				.service(fs::Files::new("/", "./static").index_file("index.html")) // static files
		})
		.disable_signals()
		.bind((std::net::Ipv4Addr::UNSPECIFIED, port)) // 0.0.0.0:port
		.unwrap()
		.shutdown_timeout(1)
		.start();

		let _ = tx.send(webserver_addr);
		let _ = sys.run();
	});
	rx.recv().unwrap()
}
