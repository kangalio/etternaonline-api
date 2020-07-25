pub use crate::common::structs::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SongData {
	pub songkey: String, // I have no idea what key is this. It has no prefix (??)
	pub id: u32,
	pub name: String,
	pub subtitle: Option<String>,
	pub author: String,
	pub artist: String,
	pub banner_url: String,
	pub background_url: String,
	pub cdtitle: Option<String>,
	pub charts: Vec<SongChartData>,
	pub packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SongChartData {
	pub chartkey: String,
	pub msd: f32,
	pub difficulty: Difficulty,
	pub is_blacklisted: bool,
	pub leaderboard: Vec<SongChartLeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SongChartLeaderboardEntry {
	pub username: String,
	pub wifescore: Wifescore,
	pub ssr_overall: f32,
	pub rate: Rate,
	pub datetime: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChartLeaderboardEntry {
	pub ssr: ChartSkillsets,
	pub wifescore: Wifescore,
	pub max_combo: u32,
	pub is_valid: bool,
	pub modifiers: String,
	pub judgements: Judgements,
	pub datetime: String,
	pub has_chord_cohesion: bool,
	pub rate: Rate,
	pub user: User,
	pub replay: Option<Replay>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct User {
	pub username: String,
	pub avatar: String,
	pub country_code: String,
	pub rating: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LatestScore {
	pub song_name: String,
	pub rate: Rate,
	pub ssr_overall: f32,
	pub wifescore: Wifescore,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UserData {
	pub user_name: String,
	pub about_me: String,
	pub country_code: String,
	pub is_moderator: bool,
	pub avatar: String,
	pub default_modifiers: Option<String>,
	pub rating: UserSkillsets,
	pub is_patreon: bool,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TopScore {
	pub song_name: String,
	pub rate: Rate,
	pub ssr_overall: f32,
	pub wifescore: Wifescore,
	pub chartkey: String,
	pub scorekey: String,
	pub difficulty: Difficulty,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CountryLeaderboardEntry {
	pub username: String,
	pub avatar: String,
	pub rating: UserSkillsets,
	pub country_code: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreData {
	pub ssr: ChartSkillsets,
	pub wifescore: Wifescore,
	pub max_combo: u32,
	pub is_valid: bool,
	pub modifiers: String,
	pub judgements: Judgements,
	pub datetime: String,
	pub has_chord_cohesion: bool,
	pub rate: Rate,
	pub user: User,
	pub replay: Option<Replay>,
	pub song: Song,
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Song {
	pub name: String,
	pub artist: String,
	pub id: u32,
}