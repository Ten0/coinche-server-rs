use coinche::*;

#[test]
fn test_clients() {
	logging::init_logger().unwrap();
	server::start(3000);
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
