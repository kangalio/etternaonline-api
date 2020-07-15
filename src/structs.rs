use thiserror::Error;

/// Skillset information. Used for player ratings, score specific ratings or difficulty
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skillsets {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64,
}

impl Skillsets {
	/// Return the overall skillset, as derived from the 7 individual skillsets
	pub fn overall(&self) -> f64 {
		(self.stream
			+ self.jumpstream
			+ self.handstream
			+ self.stamina
			+ self.jackspeed
			+ self.chordjack
			+ self.technical)
			/ 7.0
	}
}

/// Skillsets enum, excluding overall
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Skillset {
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

impl Skillset {
	/// Converts user input into a skillset variant, case-insensitively. Most community-accepted
	/// spellings of the skillsets are recognized.
	/// 
	/// Returns `None` If the given user input can't be parsed.
	/// 
	/// # Example
	/// ```rust
	/// # use etternaonline_api::Skillset;
	/// assert_eq!(Some(Skillset::Jumpstream), Skillset::from_user_input("js"));
	/// assert_eq!(Some(Skillset::Jackspeed), Skillset::from_user_input("Jacks"));
	/// assert_eq!(Some(Skillset::Jackspeed), Skillset::from_user_input("JACKSPEED"));
	/// assert_eq!(None, Skillset::from_user_input("handstreams"));
	/// ```
	pub fn from_user_input(input: &str) -> Option<Self> {
		match &input.to_lowercase() as &str {
			"stream" => Some(Skillset::Stream),
			"js" | "jumpstream" => Some(Skillset::Jumpstream),
			"hs" | "handstream" => Some(Skillset::Handstream),
			"stam" | "stamina" => Some(Skillset::Stamina),
			"jack" | "jacks" | "jackspeed" => Some(Skillset::Jackspeed),
			"cj" | "chordjack" | "chordjacks" => Some(Skillset::Chordjack),
			"tech" | "technical" => Some(Skillset::Technical),
			_ => None,
		}
	}
}

impl std::fmt::Display for Skillset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }

}

/// Chart difficulty enum
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Difficulty {
	Beginner, Easy, Medium, Hard, Challenge, Edit
}

/// Number of judgements on a score
#[derive(Debug, Eq, PartialEq, Clone, Default, Hash)]
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