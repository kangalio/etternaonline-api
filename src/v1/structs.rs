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
	pub charts: Vec<ChartData>,
	pub packs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartData {
	pub chartkey: String,
	pub msd: f32,
	pub difficulty: Difficulty,
	pub is_blacklisted: bool,
	pub leaderboard: Vec<ChartLeaderboardEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartLeaderboardEntry {
	pub username: String,
	pub wifescore: f32,
	pub ssr_overall: f32,
	pub rate: f32,
	pub datetime: String,
}