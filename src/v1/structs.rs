#[doc(inline)]
pub use crate::common::structs::*;

use etterna::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct SongData {
	pub songkey: String, // I have no idea what key is this. It has no prefix (??)
	pub id: u32,
	pub name: String,
	pub subtitle: Option<String>,
	pub author: Option<String>,
	pub artist: String,
	pub banner_url: Option<String>,
	pub background_url: Option<String>,
	pub cdtitle: Option<String>,
	pub charts: Vec<SongChartData>,
	pub packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct SongChartData {
	pub chartkey: Chartkey,
	pub msd: f32,
	pub difficulty: Difficulty,
	pub is_blacklisted: bool,
	pub leaderboard: Vec<SongChartLeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct SongChartLeaderboardEntry {
	pub username: String,
	pub wifescore: Wifescore,
	pub ssr_overall: f32,
	pub rate: Rate,
	pub datetime: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct PackEntry {
	pub id: u32,
	pub name: String,
	pub average_msd: f32,
	pub date_added: String,
	pub download_link: String,
	pub download_link_mirror: String,
	pub size: FileSize,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct ChartLeaderboardEntry {
	pub ssr: Skillsets8,
	pub wifescore: Wifescore,
	pub max_combo: u32,
	pub is_valid: bool,
	pub modifiers: String,
	pub judgements: FullJudgements,
	pub datetime: String,
	pub has_chord_cohesion: bool,
	pub rate: Rate,
	pub user: User,
	pub replay: Option<Replay>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct User {
	pub username: String,
	pub avatar: String,
	pub country_code: Option<String>,
	pub rating: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct LatestScore {
	pub song_name: String,
	pub rate: Rate,
	pub ssr_overall: f32,
	pub wifescore: Wifescore,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct UserData {
	pub user_name: String,
	pub about_me: Option<String>,
	pub country_code: Option<String>,
	pub is_moderator: bool,
	pub avatar: String,
	pub default_modifiers: Option<String>,
	pub rating: Skillsets8,
	pub is_patreon: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct TopScore {
	pub song_name: String,
	pub rate: Rate,
	pub ssr_overall: f32,
	pub wifescore: Wifescore,
	pub chartkey: Chartkey,
	pub scorekey: Scorekey,
	pub difficulty: Difficulty,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct LeaderboardEntry {
	pub username: String,
	pub avatar: String,
	pub rating: Skillsets8,
	pub country_code: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct ScoreData {
	pub ssr: Skillsets8,
	pub wifescore: Wifescore,
	pub max_combo: u32,
	pub is_valid: bool,
	pub modifiers: String,
	pub judgements: FullJudgements,
	pub datetime: String,
	pub has_chord_cohesion: bool,
	pub rate: Rate,
	pub user: User,
	pub replay: Option<Replay>,
	pub song: Song,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
#[cfg_attr(
	feature = "serde",
	derive(serde::Serialize, serde::Deserialize),
	serde(crate = "serde_")
)]
pub struct Song {
	pub name: String,
	pub artist: String,
	pub id: u32,
}
