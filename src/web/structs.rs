#[doc(inline)]
pub use crate::common::structs::*;

use etterna::*;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
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
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct LeaderboardEntry {
	pub rank: u32,
	pub username: String,
	pub country: Option<Country>,
	pub avatar: String,
	pub rating: Skillsets8,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct Country {
	pub code: String,
	pub name: String,
}

pub struct UserScores {
	/// Number of scores matching selected criteria except search query
	pub entries_before_search_filtering: u32,
	/// Number of scores matching all criteria including search query
	pub entries_after_search_filtering: u32,
	/// The range of scores that was requested
	pub scores: Vec<UserScore>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct UserScore {
	pub song_name: String,
	pub song_id: u32,
	/// This is data that is only present if the score is valid. You can also check score validity
	/// by calling `user_score.validity_dependant.is_some()`
	pub validity_dependant: Option<ValidUserScoreInfo>,
	pub rate: Rate,
	pub wifescore: Wifescore,
	pub judgements: TapJudgements,
	pub date: String,
	pub has_chord_cohesion: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
// The part of a [`UserScore`] that is only present if the score is valid
pub struct ValidUserScoreInfo {
	pub user_id: u32,
	pub ssr: Skillsets8,
	pub ssr_overall_nerfed: f32,
	pub scorekey: Scorekey,
}

impl ValidUserScoreInfo {
	pub fn nerf_factor(&self) -> f32 {
		self.ssr_overall_nerfed / self.ssr.overall
	}

	pub fn nerfed_ssr(&self) -> Skillsets8 {
		let nerf_factor = self.nerf_factor();
		Skillsets8 {
			overall: self.ssr_overall_nerfed,
			stream: self.ssr.stream * nerf_factor,
			jumpstream: self.ssr.jumpstream * nerf_factor,
			handstream: self.ssr.handstream * nerf_factor,
			stamina: self.ssr.stamina * nerf_factor,
			jackspeed: self.ssr.jackspeed * nerf_factor,
			chordjack: self.ssr.chordjack * nerf_factor,
			technical: self.ssr.technical * nerf_factor,
		}
	}
}

// TODO: I should, like, add more things to this...
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct UserDetails {
	pub user_id: u32,
	// pub total_scores: u32,
	// pub unique_songs: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub enum UserScoresSortBy {
	SongName,
	Rate,
	SsrOverall,
	SsrOverallNerfed,
	Wifescore,
	Date,
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jacks,
	Chordjacks,
	Technical,
	ChordCohesion,
	Scorekey,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub enum SortDirection {
	Descending,
	Ascending,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct ChartLeaderboard {
	/// Number of scores matching selected criteria except search query
	pub entries_before_search_filtering: u32,
	/// Number of scores matching all criteria including search query
	pub entries_after_search_filtering: u32,
	/// Requested subset of the leaderboard entries
	pub entries: Vec<ChartLeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct ChartLeaderboardEntry {
	pub username: String,
	pub scorekey: Scorekey,
	pub user_id: u32,
	pub ssr_overall: f32,
	pub ssr_overall_nerfed: f32,
	pub rate: Rate,
	pub wifescore: Wifescore,
	pub date: String,
	pub judgements: TapJudgements,
	pub max_combo: u32,
}

impl ChartLeaderboardEntry {
	/// Generate a link to this score's score page
	pub fn score_link(&self) -> String {
		format!(
			"https://etternaonline.com/score/view/{}{}",
			self.scorekey, self.user_id
		)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub enum ChartLeaderboardSortBy {
	Username,
	SsrOverall,
	Rate,
	Wife,
	Date,
	MaxCombo,
	Scorekey,
	Marvelouses,
	Perfects,
	Greats,
	Goods,
	Bads,
	Misses,
}
