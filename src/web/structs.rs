#[doc(no_inline)]
pub use crate::common::structs::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct LeaderboardEntry {
	pub rank: u32,
	pub username: String,
	pub country: Option<Country>,
	pub avatar: String,
	pub rating: UserSkillsets,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct Country {
	pub code: String,
	pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct UserScore {
	pub song_name: String,
	pub song_id: u32,
	pub scorekey: Scorekey,
	pub user_id_and_ssr: Option<ValidUserScoreInfo>,
	pub rate: Rate,
	pub wifescore: Wifescore,
	pub judgements: TapJudgements,
	pub date: String,
	pub has_chord_cohesion: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
// The part of a [`UserScore`] that is only present if the score is valid
pub struct ValidUserScoreInfo {
	pub user_id: u32,
	pub ssr: ChartSkillsets,
	pub ssr_overall_nerfed: f32,
}

// I should, like, add more things to this...
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct UserDetails {
	pub user_id: u32,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub enum UserScoresSortBy {
	SongName, Rate, SsrOverall, Wifescore, NerfedWifescore, Date,
	Stream, Jumpstream, Handstream, Stamina, Jacks, Chordjacks, Technical,
	ChordCohesion, Scorekey,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub enum SortDirection {
	Descending, Ascending,
}