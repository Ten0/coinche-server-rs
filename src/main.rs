use coinche::*;

use std::env;

#[actix_web::main]
async fn main() {
	logging::init_logger().expect("Failed to initialize logger");
	let port: u16 = env::var("PORT")
		.ok()
		.map_or(3000, |p| p.parse().expect("Invalid port value in env var"));
	server::start(port).await;
}
