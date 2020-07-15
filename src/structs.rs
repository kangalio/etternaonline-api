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