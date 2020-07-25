use std::convert::{TryFrom, TryInto};

mod calc_rating {
	fn erfc(x: f32) -> f32 { libm::erfc(x as f64) as f32 }
	
	fn is_rating_okay(rating: f32, ssrs: &[f32], delta_multiplier: f32) -> bool {
		let max_power_sum = 2f32.powf(rating / 10.0);
		
		let power_sum: f32 = ssrs.iter()
				.map(|&ssr| 2.0 / erfc(delta_multiplier * (ssr - rating)) - 2.0)
				.filter(|&x| x > 0.0)
				.sum();
		
		power_sum < max_power_sum
	}
	
	/*
	The idea is the following: we try out potential skillset rating values
	until we've found the lowest rating that still fits (I've called that
	property 'okay'-ness in the code).
	How do we know whether a potential skillset rating fits? We give each
	score a "power level", which is larger when the skillset rating of the
	specific score is high. Therefore, the user's best scores get the
	highest power levels.
	Now, we sum the power levels of each score and check whether that sum
	is below a certain limit. If it is still under the limit, the rating
	fits (is 'okay'), and we can try a higher rating. If the sum is above
	the limit, the rating doesn't fit, and we need to try out a lower
	rating.
	*/

	fn calc_rating(
		ssrs: &[f32],
		num_iters: u32,
		add_res_x2: bool,
		final_multiplier: f32,
		delta_multiplier: f32, // no idea if this is a good name
	) -> f32 {
		let mut rating: f32 = 0.0;
		let mut resolution: f32 = 10.24;
		
		// Repeatedly approximate the final rating, with better resolution
		// each time
		for _ in 0..num_iters {
			// Find lowest 'okay' rating with certain resolution
			while !is_rating_okay(rating + resolution, ssrs, delta_multiplier) {
				rating += resolution;
			}
			
			// Now, repeat with smaller resolution for better approximation
			resolution /= 2.0;
		}
		
		if add_res_x2 {
			rating += resolution * 2.0;
		}
		rating * final_multiplier
	}

	// pub fn idk_this_was_previously(ssrs: &[f32]) -> f32 {
	// 	// not sure if these params are correct; I didn't test them because I don't wannt spend the
	// 	// time and effort to find the old C++ implementation to compare
	// 	calc_rating(ssrs, 10, false, 1.04, 0.1)
	// }

	pub fn calculate_chart_overall(skillsets: &[f32]) -> f32 {
		calc_rating(skillsets, 11, true, 1.11, 0.25)
	}

	pub fn calculate_player_overall(skillsets: &[f32]) -> f32 {
		calc_rating(skillsets, 11, true, 1.0, 0.1)
	}

	// not needed rn
	// pub fn calculate_player_skillset_rating(skillsets: &[f32]) -> f32 {
	// 	calc_rating(skillsets, 11, true, 1.0, 0.1)
	// }
}

/// Skillset information. Used for chart specific difficulty, i.e. MSD and SSR
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChartSkillsets {
	pub stream: f32,
	pub jumpstream: f32,
	pub handstream: f32,
	pub stamina: f32,
	pub jackspeed: f32,
	pub chordjack: f32,
	pub technical: f32,
}
crate::impl_get8!(ChartSkillsets, f32, a, a.overall());

impl ChartSkillsets {
	/// Return the overall skillset, as derived from the 7 individual skillsets
	pub fn overall(&self) -> f32 {
		let aggregated_skillsets = calc_rating::calculate_chart_overall(&[
			self.stream,
			self.jumpstream,
			self.handstream,
			self.stamina,
			self.jackspeed,
			self.chordjack,
			self.technical,
		]);
		let max_skillset = self.stream
			.max(self.jumpstream)
			.max(self.handstream)
			.max(self.stamina)
			.max(self.jackspeed)
			.max(self.chordjack)
			.max(self.technical);
		
		aggregated_skillsets.max(max_skillset)
	}
}

/// Skillset information. Used for player ratings
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserSkillsets {
	pub stream: f32,
	pub jumpstream: f32,
	pub handstream: f32,
	pub stamina: f32,
	pub jackspeed: f32,
	pub chordjack: f32,
	pub technical: f32,
}
crate::impl_get8!(UserSkillsets, f32, a, a.overall());

impl UserSkillsets {
	/// Return the overall skillset, as derived from the 7 individual skillsets
	pub fn overall(&self) -> f32 {
		calc_rating::calculate_player_overall(&[
			self.stream,
			self.jumpstream,
			self.handstream,
			self.stamina,
			self.jackspeed,
			self.chordjack,
			self.technical,
		])
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

	pub fn into_skillset8(self) -> Skillset8 {
		match self {
			Self::Stream => Skillset8::Stream,
			Self::Jumpstream => Skillset8::Jumpstream,
			Self::Handstream => Skillset8::Handstream,
			Self::Stamina => Skillset8::Stamina,
			Self::Jackspeed => Skillset8::Jackspeed,
			Self::Chordjack => Skillset8::Chordjack,
			Self::Technical => Skillset8::Technical,
		}
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

	pub fn into_skillset7(self) -> Option<Skillset7> {
		match self {
			Self::Overall => None,
			Self::Stream => Some(Skillset7::Stream),
			Self::Jumpstream => Some(Skillset7::Jumpstream),
			Self::Handstream => Some(Skillset7::Handstream),
			Self::Stamina => Some(Skillset7::Stamina),
			Self::Jackspeed => Some(Skillset7::Jackspeed),
			Self::Chordjack => Some(Skillset7::Chordjack),
			Self::Technical => Some(Skillset7::Technical),
		}
	}
}

impl TryFrom<Skillset8> for Skillset7 {
	type Error = ();

	fn try_from(ss: Skillset8) -> Result<Skillset7, ()> {
		ss.into_skillset7().ok_or(())
	}
}

impl std::convert::From<Skillset7> for Skillset8 {
	fn from(ss: Skillset7) -> Skillset8 {
		ss.into_skillset8()
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