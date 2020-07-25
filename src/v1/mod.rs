mod structs;
pub use structs::*;

use crate::Error;
use crate::extension_traits::*;
use crate::structs::*;


pub struct Session {
	api_key: String,
	cooldown: std::time::Duration,
	timeout: Option<std::time::Duration>,
	last_request: std::time::Instant,
}

impl Session {
	pub fn new(
		api_key: String,
		cooldown: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Self {
		Self { api_key, cooldown, timeout, last_request: std::time::Instant::now() - cooldown }
	}

	fn request(&mut self, path: &str, parameters: &[(&str, &str)]) -> Result<serde_json::Value, Error> {
		crate::rate_limit(&mut self.last_request, self.cooldown);

		let mut request = ureq::get(&format!("https://api.etternaonline.com/v1/{}", path));
		for (param, value) in parameters {
			request.query(param, value);
		}
		request.query("api_key", &self.api_key);
		if let Some(timeout) = self.timeout {
			request.timeout(timeout);
		}

		let response = request.call();

		let json = response.into_json().map_err(|e| Error::InvalidJson(e.to_string()))?;
		Ok(json)
	}

	pub fn song_data(&mut self, song_id: u32) -> Result<SongData, Error> {
		let json = self.request("song", &[("key", song_id.to_string().as_str())])?;
		let json = match &json.array()?.as_slice() {
			&[x] => x,
			other => return Err(Error::InvalidJsonStructure(
				Some(format!("Expected one array element, found {}", other.len()))
			)),
		};

		Ok(SongData {
			songkey: json["songkey"].string()?,
			id: json["id"].u32_string()?,
			name: json["songname"].string()?,
			subtitle: json["subtitle"].string_maybe()?,
			author: json["author"].string()?,
			artist: json["artist"].string()?,
			banner_url: json["banner"].string()?,
			background_url: json["banner"].string()?,
			cdtitle: json["cdtitle"].string_maybe()?,
			charts: json["charts"].array()?.iter().map(|json| Ok(SongChartData {
				chartkey: json["chartkey"].string()?,
				msd: json["msd"].f32_string()?,
				difficulty: crate::Difficulty::from_long_string(json["difficulty"].str_()?).json_unwrap()?,
				is_blacklisted: json["blacklisted"].bool_int_string()?,
				leaderboard: json["leaderboard"].array()?.iter().map(|json| Ok(SongChartLeaderboardEntry {
					username: json["username"].string()?,
					wifescore: json["wifescore"].f32_string()?,
					ssr_overall: json["Overall"].f32_string()?,
					rate: json["user_chart_rate_rate"].f32_string()?,
					datetime: json["datetime"].string()?,
				})).collect::<Result<Vec<SongChartLeaderboardEntry>, Error>>()?,
			})).collect::<Result<Vec<SongChartData>, Error>>()?,
			packs: json["packs"].array()?.iter().map(|v| Ok(
				v.string()?
			)).collect::<Result<Vec<String>, Error>>()?,
		})
	}

	pub fn client_version(&mut self) -> Result<String, Error> {
		Ok(self.request("clientVersion", &[])?["version"].string()?)
	}

	/// Retrieve the link where you can register for an EO account
	pub fn register_link(&mut self) -> Result<String, Error> {
		Ok(self.request("registerLink", &[])?["link"].string()?)
	}

	pub fn pack_list(&mut self) -> Result<Vec<PackEntry>, Error> {
		self.request("pack_list", &[])?.array()?.iter().map(|json| Ok(PackEntry {
			id: json["packid"].u32_()?,
			name: json["packname"].string()?,
			average_msd: json["average"].f32_()?,
			date_added: json["date"].string()?,
			download_link: json["download"].string()?,
			download_link_mirror: json["mirror"].string()?,
			size: FileSize::from_bytes(json["size"].u64_()?),
		})).collect()
	}

	pub fn chart_leaderboard(&mut self, chartkey: &str) -> Result<Vec<ChartLeaderboardEntry>, Error> {
		let json = self.request("chartLeaderboard", &[("chartkey", chartkey)])?;
		json.array()?.iter().map(|json| Ok(ChartLeaderboardEntry {
			ssr: ChartSkillsets {
				stream: json["Stream"].f32_string()?,
				jumpstream: json["Jumpstream"].f32_string()?,
				handstream: json["Handstream"].f32_string()?,
				stamina: json["Stamina"].f32_string()?,
				jackspeed: json["JackSpeed"].f32_string()?,
				chordjack: json["Chordjack"].f32_string()?,
				technical: json["Technical"].f32_string()?,
			},
			wifescore: json["wifescore"].f32_string()?,
			max_combo: json["maxcombo"].u32_string()?,
			is_valid: json["valid"].bool_int_string()?,
			modifiers: json["modifiers"].string()?,
			judgements: Judgements {
				marvelouses: json["marv"].u32_string()?,
				perfects: json["perfect"].u32_string()?,
				greats: json["great"].u32_string()?,
				goods: json["good"].u32_string()?,
				bads: json["bad"].u32_string()?,
				misses: json["miss"].u32_string()?,
				hit_mines: json["hitmine"].u32_string()?,
				held_holds: json["held"].u32_string()?,
				let_go_holds: json["letgo"].u32_string()?,
				missed_holds: json["missedhold"].u32_string()?,
			},
			datetime: json["datetime"].string()?,
			has_chord_cohesion: !json["nocc"].bool_int_string()?,
			rate: json["user_chart_rate_rate"].f32_string()?,
			user: User {
				username: json["username"].string()?,
				avatar: json["avatar"].string()?,
				country_code: json["countryCode"].string()?,
				rating: json["player_rating"].f32_string()?,
			},
			replay: crate::parse_replay(&json["replay"])?,
		})).collect()
	}
}