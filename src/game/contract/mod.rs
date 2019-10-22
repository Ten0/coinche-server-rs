pub mod bid;
pub mod trump;

#[derive(Serialize)]
pub struct Contract {
	pub points: usize,
	pub trump: Trump,
}

pub use bid::*;
pub use trump::*;
