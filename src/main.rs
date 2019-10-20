use coinche::*;

fn main() -> coinche::Result<()> {
	server::start_server();

	std::thread::spawn(|| client("User1"));
	std::thread::sleep(std::time::Duration::from_millis(100));
	std::thread::spawn(|| client("User2"));

	std::thread::sleep(std::time::Duration::from_secs(1));
	Ok(())
}

fn client(username: &str) {
	use ws::connect;
	if let Err(error) = connect("ws://127.0.0.1:3000", |out| {
		// Queue a message to be sent when the WebSocket is open
		if let Err(_) = out.send(username) {
			println!("Websocket couldn't queue an initial message.")
		} else {
			println!("Client sent message 'Hello WebSocket'. ")
		}

		// The handler needs to take ownership of out, so we use move
		move |msg| {
			// Handle messages received on this connection
			println!("Client got message '{}'. ", msg);

			// Close the connection
			//out.close(CloseCode::Normal)
			Ok(())
		}
	}) {
		// Inform the user of failure
		println!("Failed to create WebSocket due to: {:?}", error);
	}
}
