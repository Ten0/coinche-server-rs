use coinche::*;
use futures::Future;
use std::env;
use std::sync::mpsc;

fn main() {
	websocket::start_server(env::args().nth(1).map_or(false, |s| s == "-v"));
	println!("WebSocket server listening on 0.0.0.0:3000");
	let static_server = static_files::start_server();
	println!("Static files server listening on 0.0.0.0:3001");
	let (stop_s, stop_r) = mpsc::channel();
	ctrlc::set_handler(move || {
		stop_s.send(()).unwrap();
	})
	.expect("Error setting Ctrl-C handler");
	stop_r.recv().unwrap();
	static_server.stop(true).wait().unwrap();
}
