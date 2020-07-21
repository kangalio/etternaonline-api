use std::convert::{TryFrom, TryInto};

/// Skillset information. Used for player ratings, score specific ratings or difficulty
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChartSkillsets {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64,
}
crate::impl_get8!(ChartSkillsets, f64, a, a.overall());

impl ChartSkillsets {
	/// Return the overall skillset, as derived from the 7 individual skillsets
	pub fn overall(&self) -> f64 {
		self.stream
			.max(self.jumpstream)
			.max(self.handstream)
			.max(self.stamina)
			.max(self.jackspeed)
			.max(self.chordjack)
			.max(self.technical)
	}
}

/// Skillset information. Used for player ratings, score specific ratings or difficulty
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserSkillsets {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64,
}
crate::impl_get8!(UserSkillsets, f64, a, a.overall());

impl UserSkillsets {
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Skillset7 {
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

impl Skillset7 {
	/// Same as [`Skillset8::from_user_input`]
	pub fn from_user_input(input: &str) -> Option<Self> {
		match Skillset8::from_user_input(input) {
			Some(skillset) => skillset.try_into().ok(),
			None => None,
		}
	}

	/// Iterate over all skillsets
	pub fn iter() -> impl Iterator<Item=Self> {
		[Self::Stream, Self::Jumpstream, Self::Handstream, Self::Stamina, Self::Jackspeed,
			Self::Chordjack, Self::Technical].iter().copied()
	}
}

/// Skillsets enum, including overall
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Skillset8 {
	Overall,
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

impl Skillset8 {
	/// Converts user input into a skillset variant, case-insensitively. Most community-accepted
	/// spellings of the skillsets are recognized.
	/// 
	/// Returns `None` If the given user input can't be parsed.
	/// 
	/// # Example
	/// ```rust
	/// # use etternaonline_api::Skillset8;
	/// assert_eq!(Some(Skillset8::Jumpstream), Skillset8::from_user_input("js"));
	/// assert_eq!(Some(Skillset8::Jackspeed), Skillset8::from_user_input("Jacks"));
	/// assert_eq!(Some(Skillset8::Jackspeed), Skillset8::from_user_input("JACKSPEED"));
	/// assert_eq!(None, Skillset8::from_user_input("handstreams"));
	/// ```
	pub fn from_user_input(input: &str) -> Option<Self> {
		match &input.to_lowercase() as &str {
			"overall" => Some(Self::Overall),
			"stream" => Some(Self::Stream),
			"js" | "jumpstream" => Some(Self::Jumpstream),
			"hs" | "handstream" => Some(Self::Handstream),
			"stam" | "stamina" => Some(Self::Stamina),
			"jack" | "jacks" | "jackspeed" => Some(Self::Jackspeed),
			"cj" | "chordjack" | "chordjacks" => Some(Self::Chordjack),
			"tech" | "technical" => Some(Self::Technical),
			_ => None,
		}
	}

	/// Iterate over all skillsets
	pub fn iter() -> impl Iterator<Item=Self> {
		[Self::Overall, Self::Stream, Self::Jumpstream, Self::Handstream, Self::Stamina,
			Self::Jackspeed, Self::Chordjack, Self::Technical].iter().copied()
	}
}

impl TryFrom<Skillset8> for Skillset7 {
	type Error = ();

	fn try_from(ss: Skillset8) -> Result<Skillset7, ()> {
		match ss {
			Skillset8::Overall => Err(()),
			Skillset8::Stream => Ok(Self::Stream),
			Skillset8::Jumpstream => Ok(Self::Jumpstream),
			Skillset8::Handstream => Ok(Self::Handstream),
			Skillset8::Stamina => Ok(Self::Stamina),
			Skillset8::Jackspeed => Ok(Self::Jackspeed),
			Skillset8::Chordjack => Ok(Self::Chordjack),
			Skillset8::Technical => Ok(Self::Technical),
		}
	}
}

impl std::convert::From<Skillset7> for Skillset8 {
	fn from(ss: Skillset7) -> Skillset8 {
		match ss {
			Skillset7::Stream => Self::Stream,
			Skillset7::Jumpstream => Self::Jumpstream,
			Skillset7::Handstream => Self::Handstream,
			Skillset7::Stamina => Self::Stamina,
			Skillset7::Jackspeed => Self::Jackspeed,
			Skillset7::Chordjack => Self::Chordjack,
			Skillset7::Technical => Self::Technical,
		}
	}
}

impl std::fmt::Display for Skillset7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for Skillset8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}