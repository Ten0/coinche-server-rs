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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_clients() {
		websocket::start_server(false);
		let _ = client("c1");
	}

	fn client(username: &str) {
		use ws::connect;
		if let Err(error) = connect("ws://127.0.0.1:3000", |out| {
			// Queue a message to be sent when the WebSocket is open
			if let Err(_) = out.send(format!(r#"{{"Init": {{"username": "{}"}}}}"#, username)) {
				panic!("Websocket couldn't queue an initial message.")
			} else {
				println!("Client sent message.")
			}

			// The handler needs to take ownership of out, so we use move
			move |msg| {
				// Handle messages received on this connection
				println!("Client got message '{}'. ", msg);

				// Close the connection
				out.close(ws::CloseCode::Normal)?;
				Ok(())
			}
		}) {
			// Inform the user of failure
			panic!("Failed to create WebSocket due to: {:?}", error);
		}
	}
}
