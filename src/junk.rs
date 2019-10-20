pub enum Socket {
	Unitnitialized { game: GameArc, sender: Sender },
	Initialized { player: PlayerArc },
}
impl Socket {
	fn send(&self, msg: impl Into<ws::Message>) -> ws::Result<()> {
		self.sender().send(msg)
	}

	fn sender(&self) -> &Sender {
		match self {
			Unitnitialized { sender, .. } => &sender,
			Initialized { player } => player.game,
		}
	}
}
