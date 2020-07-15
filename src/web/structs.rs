pub use crate::structs::*;

#[derive(Debug)]
pub struct PackEntry {
	pub name: String,
	pub id: u32,
	pub datetime: String,
	pub size: FileSize,
	pub average_msd: f64,
	pub num_votes: u32,
	pub average_vote: f64,
	pub download_link: String,
}