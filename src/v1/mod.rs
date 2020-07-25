mod structs;
pub use structs::*;

use crate::Error;
use crate::extension_traits::*;


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
			charts: json["charts"].array()?.iter().map(|json| Ok(ChartData {
				chartkey: json["chartkey"].string()?,
				msd: json["msd"].f32_string()?,
				difficulty: crate::Difficulty::from_long_string(json["difficulty"].str_()?).json_unwrap()?,
				is_blacklisted: json["blacklisted"].bool_int_string()?,
				leaderboard: json["leaderboard"].array()?.iter().map(|json| Ok(ChartLeaderboardEntry {
					username: json["username"].string()?,
					wifescore: json["wifescore"].f32_string()?,
					ssr_overall: json["Overall"].f32_string()?,
					rate: json["user_chart_rate_rate"].f32_string()?,
					datetime: json["datetime"].string()?,
				})).collect::<Result<Vec<ChartLeaderboardEntry>, Error>>()?,
			})).collect::<Result<Vec<ChartData>, Error>>()?,
			packs: json["packs"].array()?.iter().map(|v| Ok(
				v.string()?
			)).collect::<Result<Vec<String>, Error>>()?,
		})
	}
}