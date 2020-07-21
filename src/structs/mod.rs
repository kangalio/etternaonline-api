mod skillsets;
pub use skillsets::*;

use thiserror::Error;

/// Chart difficulty enum
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Difficulty {
	Beginner, Easy, Medium, Hard, Challenge, Edit
}

/// Number of judgements on a score
#[derive(Debug, Eq, PartialEq, Clone, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Judgements {
	pub marvelouses: u32,
	pub perfects: u32,
	pub greats: u32,
	pub goods: u32,
	pub bads: u32,
	pub misses: u32,
	pub hit_mines: u32,
	pub held_holds: u32,
	pub let_go_holds: u32,
	pub missed_holds: u32,
}

/// Type of a note
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NoteType {
	Tap,
	HoldHead,
	HoldTail,
	Mine,
	Lift,
	Keysound,
	Fake,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileSize {
	bytes: u64,
}

impl FileSize {
	pub fn bytes(self) -> u64 { self.bytes }
	pub fn kb(self) -> u64 { self.bytes / 1_000 }
	pub fn mb(self) -> u64 { self.bytes / 1_000_000 }
	pub fn gb(self) -> u64 { self.bytes / 1_000_000_000 }
	pub fn tb(self) -> u64 { self.bytes / 1_000_000_000_000 }
}

#[derive(Debug, Error)]
pub enum FileSizeParseError {
	#[error("Given string was empty")]
	EmptyString,
	#[error("Error while parsing the filesize number")]
	InvalidNumber(#[source] std::num::ParseFloatError),
	#[error("No KB/MB/... ending")]
	NoEnding,
	#[error("Unknown ending (i.e. the KB/MB/... thingy)")]
	UnexpectedEnding(String),
}

impl FileSize {
	pub fn from_bytes(bytes: u64) -> Self {
		Self { bytes }
	}
}

impl std::str::FromStr for FileSize {
	type Err = FileSizeParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut token_iter = s.split_whitespace();
		let number: f64 = token_iter.next().ok_or(FileSizeParseError::EmptyString)?
			.parse().map_err(FileSizeParseError::InvalidNumber)?;
		let ending = token_iter.next().ok_or(FileSizeParseError::NoEnding)?;

		let ending = ending.to_lowercase();
		let multiplier: u64 = match &ending as &str {
			"b"	  => 1,
			"kb"  => 1000,
			"kib" => 1024,
			"mb"  => 1000 * 1000,
			"mib" => 1024 * 1024,
			"gb"  => 1000 * 1000 * 1000,
			"gib" => 1024 * 1024 * 1024,
			"tb"  => 1000 * 1000 * 1000 * 1000,
			"tib" => 1024 * 1024 * 1024 * 1024,
			_ => return Err(FileSizeParseError::UnexpectedEnding(ending)),
		};

		Ok(Self::from_bytes((number * multiplier as f64) as u64))
	}
}