mod skillsets;
pub use skillsets::*;

use thiserror::Error;

/// Chart difficulty enum
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Difficulty {
	Beginner, Easy, Medium, Hard, Challenge, Edit
}

impl Difficulty {
	/// Parses a short difficulty string as found on the Etterna evaluation screen: BG, IN... The
	/// string must be given in uppercase letters
	pub fn from_short_string(string: &str) -> Option<Self> {
		match string {
			"BG" => Some(Self::Beginner),
			"EZ" => Some(Self::Easy),
			"NM" => Some(Self::Medium),
			"HD" => Some(Self::Hard),
			"IN" => Some(Self::Challenge),
			"ED" => Some(Self::Edit),
			_ => None,
		}
	}

	/// Generate a short difficulty string as found on the Etterna evaluation screen.
	pub fn to_short_string(self) -> &'static str {
		match self {
			Self::Beginner => "BG",
			Self::Easy => "EZ",
			Self::Medium => "NM",
			Self::Hard => "HD",
			Self::Challenge => "IN",
			Self::Edit => "ED",
		}
	}

	pub fn from_long_string(string: &str) -> Option<Self> {
		match string {
			"Beginner" | "Novice" => Some(Self::Beginner),
			"Easy" => Some(Self::Easy),
			"Medium" | "Normal" => Some(Self::Medium),
			"Hard" => Some(Self::Hard),
			"Challenge" | "Expert" | "Insane" => Some(Self::Challenge),
			"Edit" => Some(Self::Edit),
			_ => None,
		}
	}
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

/// Replay data, contains [`ReplayNote`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Replay {
	pub notes: Vec<ReplayNote>,
}

/// A singular note, used inside [`Replay`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplayNote {
	/// The position of the note inside the chart, in seconds
	pub time: f32,
	/// The offset that the note was hit with, in seconds. A 50ms early hit would be `-0.05`
	pub deviation: f32,
	/// The position of the ntoe inside the chart, in ticks (192nds)
	pub tick: Option<u32>,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k
	pub lane: u8,
	/// Type of the note (tap, hold, mine etc.)
	pub note_type: NoteType,
}

/// Global ranks in each skillset category
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserRank {
	pub overall: u32,
	pub stream: u32,
	pub jumpstream: u32,
	pub handstream: u32,
	pub stamina: u32,
	pub jackspeed: u32,
	pub chordjack: u32,
	pub technical: u32,
}
crate::impl_get8!(UserRank, u32, a, a.overall);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Rate {
	// this value is 20x the real rate, e.g. `1.15x` would be 23
	x20: u32,
}

impl Rate {
	/// Rounds to the nearest valid rate.
	/// 
	/// Returns None if the given value is negative or too large
	pub fn from_f32(r: f32) -> Option<Self> {
		// Some(Self { x20: (r * 20.0).round().try_into().ok()? })
		if r < 0.0 || r > u32::MAX as f32 {
			None
		} else {
			Some(Self { x20: (r * 20.0).round() as u32 })
		}
	}

	/// Parses a string into a rate. The string needs to be in the format `\d+\.\d+[05]?`
	/// 
	/// Returns None if parsing failed
	pub fn from_string(string: &str) -> Option<Self> {
		// not the most efficient but /shrug
		Self::from_f32(string.parse().ok()?)
	}

	/// Create a new rate from a value that is equal to the real rate multiplied by 20.
	/// 
	/// Due to the fact that Etterna ratings are always multiples of 0.05, every rate can be
	/// precicely represented precisely with a whole number when multiplied by 20.
	pub fn from_x20(x20: u32) -> Self {
		Self { x20 }
	}
}

impl std::fmt::Display for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x", self.x20 as f32 / 20.0)
    }
}

impl std::fmt::Debug for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x", self.x20 as f32 / 20.0)
    }
}

impl Default for Rate {
    fn default() -> Self {
        Self::from_x20(20)
    }
}