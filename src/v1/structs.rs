use crate::structs::*;

#[derive(Debug, Clone, PartialEq)]
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
pub struct SongChartData {
	pub chartkey: String,
	pub msd: f32,
	pub difficulty: Difficulty,
	pub is_blacklisted: bool,
	pub leaderboard: Vec<SongChartLeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SongChartLeaderboardEntry {
	pub username: String,
	pub wifescore: f32,
	pub ssr_overall: f32,
	pub rate: f32,
	pub datetime: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
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
pub struct ChartLeaderboardEntry {
	ssr: ChartSkillsets,
	wifescore: f32,
	max_combo: u32,
	is_valid: bool,
	modifiers: String,
	judgements: Judgements,
	datetime: String,
	has_chord_cohesion: bool,
	rate: f32,
	user: User,
	replay: Option<Replay>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct User {
	username: String,
	avatar: String,
	country_code: String,
	rating: f32,
}