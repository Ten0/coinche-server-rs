use coinche::*;

use futures::Future;
use std::{env, sync::mpsc};

#[macro_use]
extern crate log;

fn main() {
	logging::init_logger().expect("Failed to initialize logger");
	let port: u16 = env::var("PORT")
		.ok()
		.map_or(3000, |p| p.parse().expect("Invalid port value in env var"));
	let server = server::start(port);
	info!("Server listening on 0.0.0.0:{}", port);
	let (stop_s, stop_r) = mpsc::channel();
	ctrlc::set_handler(move || {
		stop_s.send(()).unwrap();
	})
	.expect("Error setting Ctrl-C handler");
	stop_r.recv().unwrap();
	server.stop(true).wait().unwrap();
}
