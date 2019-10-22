pub mod websocket;

use actix_files as fs;
use actix_web::{dev::Server, middleware, web, App, HttpServer};
use std::{sync::mpsc, thread};

pub fn start(port: u16) -> Server {
	let (tx, rx) = mpsc::channel();

	thread::spawn(move || {
		let sys = actix_rt::System::new("http-server");

		let addr = HttpServer::new(|| {
			App::new()
				.wrap(middleware::Logger::default()) // Enable logger
				.wrap(middleware::Compress::default()) // Enable compression if client asks for it
				.route("/ws/", web::get().to(websocket::index)) // websocket route
				.service(fs::Files::new("/", "./static").index_file("index.html")) // static files
		})
		.bind((std::net::Ipv4Addr::UNSPECIFIED, port)) // 0.0.0.0:port
		.unwrap()
		.shutdown_timeout(1)
		.start();

		let _ = tx.send(addr);
		let _ = sys.run();
	});
	rx.recv().unwrap()
}
