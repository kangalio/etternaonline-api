#[derive(Debug, PartialEq, Clone)]
pub struct UserDetails {
	pub username: String,
	pub about_me: String,
	pub is_moderator: bool,
	pub is_patreon: bool,
	pub avatar_url: String,
	pub country_code: String,
	pub player_rating: f64,
	pub default_modifiers: String,
	pub skillsets: Skillsets8,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skillsets7 {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skillsets8 {
	pub overall: f64,
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopScore {
	/// The scorekey, for example `S65565b5bc377c6d78b60c0aecfd9e05955b4cf63`
	pub scorekey: String,
	/// The song name, for example `Game Time`
	pub song_name: String,
	/// The overall score-specific-rating
	pub ssr_overall: f64,
	/// Wifescore of the score, from 0.0 to 100.0
	pub wifescore: f64,
	/// The music rate
	pub rate: f64,
	/// The chart difficulty
	pub difficulty: Difficulty,
	/// The key of the chart, for example `X6ea10eba800cfcbfe462e902da3d3cdfb8d546d9`
	pub chartkey: String,
	/// The MSD of the chart on 1.0x
	pub base_skillsets: Skillsets7,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatestScore {
	/// The scorekey, for example `S65565b5bc377c6d78b60c0aecfd9e05955b4cf63`
	pub scorekey: String,
	/// The song name, for example `Game Time`
	pub song_name: String,
	/// The overall score-specific-rating
	pub ssr_overall: f64,
	/// Wifescore of the score, from 0.0 to 100.0
	pub wifescore: f64,
	/// The music rate
	pub rate: f64,
	/// The chart difficulty
	pub difficulty: Difficulty,

}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Skillset7 {
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
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

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Difficulty {
	Beginner, Easy, Medium, Hard, Challenge, Edit
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UserRanksPerSkillset {
	pub overall: u32,
	pub stream: u32,
	pub jumpstream: u32,
	pub handstream: u32,
	pub stamina: u32,
	pub jackspeed: u32,
	pub chordjack: u32,
	pub technical: u32,
}

// pub struct TopScorePerSkillset {
// 	pub song_name: String,
// 	pub rate: f64,
// 	pub wifescore: f64,
// 	pub chartkey: String,
// 	pub scorekey: String,
// 	pub difficulty: Difficulty,
// }

// pub struct UserTopScoresPerSkillset {
// 	pub overall: TopScorePerSkillset,
// 	pub stream: TopScorePerSkillset,
// 	pub jumpstream: TopScorePerSkillset,
// 	pub handstream: TopScorePerSkillset,
// 	pub stamina: TopScorePerSkillset,
// 	pub jackspeed: TopScorePerSkillset,
// 	pub chordjack: TopScorePerSkillset,
// 	pub technical: TopScorePerSkillset,
// }