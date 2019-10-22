pub fn init_logger() -> Result<(), fern::InitError> {
	let colors = fern::colors::ColoredLevelConfig::default().info(fern::colors::Color::Blue);
	fern::Dispatch::new()
		// Perform allocation-free log formatting
		.format(move |out, message, record| {
			out.finish(format_args!(
				"{}[{}][{}] {}",
				chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
				record.target(),
				colors.color(record.level()),
				message
			))
		})
		// Add blanket level filter -
		.level(log::LevelFilter::Info)
		// - and per-module overrides
		.level_for(env!("CARGO_PKG_NAME"), log::LevelFilter::Debug)
		.chain(std::io::stdout())
		// Apply globally
		.apply()?;
	Ok(())
}
