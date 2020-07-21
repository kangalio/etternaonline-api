pub use crate::structs::*;

/// Details about a user. See [`Session::user_details`](super::Session::user_details)
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserDetails {
	pub username: String,
	pub about_me: String,
	pub is_moderator: bool,
	pub is_patreon: bool,
	pub avatar_url: String,
	pub country_code: String,
	pub player_rating: f64,
	pub default_modifiers: Option<String>,
	pub rating: UserSkillsets,
}

/// Score from a top scores enumeration like [`Session::user_top_10_scores`](super::Session::user_top_10_scores)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TopScore {
	pub scorekey: String,
	pub song_name: String,
	pub ssr_overall: f64,
	pub wifescore: f64,
	pub rate: f64,
	pub difficulty: Difficulty,
	pub chartkey: String,
	pub base_msd: ChartSkillsets,
}

/// Score from a latest scores enumeration like [`Session::user_latest_scores`](super::Session::user_latest_scores)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LatestScore {
	pub scorekey: String,
	pub song_name: String,
	pub ssr_overall: f64,
	pub wifescore: f64,
	pub rate: f64,
	pub difficulty: Difficulty,

}

/// Global ranks in each skillset category. See [`Session::user_ranks_per_skillset`](super::Session::user_ranks_per_skillset)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
crate::impl_get8!(UserRanksPerSkillset, u32, a, a.overall);

/// Score from a [top scores per skillset enumeration](super::Session::user_top_scores_per_skillset)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TopScorePerSkillset {
	pub song_name: String,
	pub rate: f64,
	pub wifescore: f64,
	pub chartkey: String,
	pub scorekey: String,
	pub difficulty: Difficulty,
	pub ssr: ChartSkillsets,
}

/// User's best scores in each skillset category. See [`Session::user_top_scores_per_skillset`](super::Session::user_top_scores_per_skillset)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserTopScoresPerSkillset {
	pub overall: Vec<TopScorePerSkillset>,
	pub stream: Vec<TopScorePerSkillset>,
	pub jumpstream: Vec<TopScorePerSkillset>,
	pub handstream: Vec<TopScorePerSkillset>,
	pub stamina: Vec<TopScorePerSkillset>,
	pub jackspeed: Vec<TopScorePerSkillset>,
	pub chordjack: Vec<TopScorePerSkillset>,
	pub technical: Vec<TopScorePerSkillset>,
}

/// Generic information about a score
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreData {
	pub scorekey: String,
	pub modifiers: String,
	pub ssr: ChartSkillsets,
	pub wifescore: f64,
	pub rate: f64,
	pub max_combo: u32,
	pub is_valid: bool,
	pub has_chord_cohesion: bool,
	pub judgements: Judgements,
	pub replay: Option<Replay>,
	pub user: ScoreUser,
	pub song_name: String,
	pub artist: String,
	pub song_id: u32,
}

/// User information contained within a score information struct
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreUser {
	pub username: String,
	pub avatar: String,
	pub country_code: String,
	pub overall_rating: f64,
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
	pub time: f64,
	/// The offset that the note was hit with, in seconds. A 50ms early hit would be `-0.05`
	pub deviation: f64,
	/// The position of the ntoe inside the chart, in ticks (192nds)
	pub tick: u32,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k
	pub lane: u8,
	/// Type of the note (tap, hold, mine etc.)
	pub note_type: NoteType,
}

/// Score information in the context of a [chart leaderboard](super::Session::chart_leaderboard)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChartLeaderboardScore {
	pub scorekey: String,
	pub ssr: ChartSkillsets,
	pub wifescore: f64,
	pub rate: f64,
	pub max_combo: u32,
	pub is_valid: bool,
	pub has_chord_cohesion: bool,
	pub datetime: String,
	pub modifiers: String,
	pub has_replay: bool,
	pub judgements: Judgements,
	pub user: ScoreUser,
}

/// Entry in a score leaderboard
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LeaderboardEntry {
	pub user: ScoreUser,
	pub rating: UserSkillsets,
}

/// Score goal
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreGoal {
	pub chartkey: String,
	pub rate: f64,
	pub wifescore: f64,
	pub time_assigned: String,
	pub time_achieved: Option<String>,
}