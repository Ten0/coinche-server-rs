use actix_files as fs;
use actix_web::{dev::Server, middleware, App, HttpServer};
use std::{sync::mpsc, thread};

pub fn start_server() -> Server {
	let (tx, rx) = mpsc::channel();

	thread::spawn(move || {
		let sys = actix_rt::System::new("http-server");

		let addr = HttpServer::new(|| {
			App::new()
				.wrap(middleware::Compress::default())
				.service(fs::Files::new("/", "./static").index_file("index.html"))
		})
		.bind("0.0.0.0:3001")
		.unwrap()
		.shutdown_timeout(1)
		.start();

		let _ = tx.send(addr);
		let _ = sys.run();
	});
	rx.recv().unwrap()
}
