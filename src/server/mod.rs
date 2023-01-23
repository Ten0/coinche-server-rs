pub mod game;
pub mod websocket;

use crate::prelude::*;

use {
	actix::prelude::*,
	actix_files as fs,
	actix_web::{
		dev::Service,
		http::header::{self, HeaderValue},
		middleware, web, App, HttpResponse, HttpServer,
	},
	futures::future::Either,
};

pub async fn start(port: u16) {
	let game_addr = Game::new().start();

	let webserver = HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(game_addr.clone()))
			.wrap(middleware::Logger::default())
			.wrap(middleware::Compress::default())
			.wrap_fn(|req, srv| {
				// Enforce HTTPS if forwarded from http (heroku)
				let headers = req.headers();
				if headers.get("X-Forwarded-Proto").map_or(false, |v| v == "http") {
					let host_header = headers.get(header::HOST);
					let host_header_str = host_header.and_then(|h| h.to_str().ok()).unwrap_or("perdu.com");
					let location = format!("https://{}{}", host_header_str, req.path());
					Either::Left(future::ok(
						req.into_response(
							HttpResponse::MovedPermanently()
								.append_header((header::LOCATION, location))
								.finish(),
						),
					))
				} else {
					Either::Right(srv.call(req).map_ok(|mut res| {
						res.headers_mut().insert(
							header::STRICT_TRANSPORT_SECURITY,
							HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
						);
						res.map_into_boxed_body()
					}))
				}
			})
			.route("/ws/", web::get().to(websocket::index))
			.service(fs::Files::new("/", "./static").index_file("index.html"))
	})
	.bind((std::net::Ipv4Addr::UNSPECIFIED, port))
	.unwrap()
	.shutdown_timeout(1)
	.run();

	info!("Server listening on 0.0.0.0:{}", port);

	webserver.await.unwrap()
}
