mod structs;
pub use structs::*;

use crate::Error;
use crate::extension_traits::*;


fn user_skillsets_from_eo(json: &serde_json::Value) -> Result<UserSkillsets, Error> {
	Ok(UserSkillsets {
		stream: json["Stream"].f32_string()?,
		jumpstream: json["Jumpstream"].f32_string()?,
		handstream: json["Handstream"].f32_string()?,
		stamina: json["Stamina"].f32_string()?,
		jackspeed: json["JackSpeed"].f32_string()?,
		chordjack: json["Chordjack"].f32_string()?,
		technical: json["Technical"].f32_string()?,
	})
}

/// EtternaOnline API session client, handles all requests to and from EtternaOnline.
/// 
/// This handler has rate-limiting built-in. Please do make use of it - the EO server is brittle and
/// funded entirely by donations.
/// 
/// Initialize a session using [`Session::new`]
/// 
/// # Example
/// ```rust
/// let mut session = Session::new(
/// 	"<API KEY HERE>"
/// 	std::time::Duration::from_millis(2000), // Wait 2s inbetween requests
/// 	None, // No request timeout
/// );
/// 
/// println!("Details about kangalioo: {:?}", session.user_data("kangalioo")?);
/// 
/// let best_score = session.user_top_scores("kangalioo", Skillset8::Overall, 1)?[0];
/// println!(
/// 	"kangalioo's best score has {} misses",
/// 	session.score_data(best_score)?.judgements.misses
/// );
/// ```
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

		if let Some(error) = json["error"].as_str() {
			return Err(match error {
				"Chart not tracked" => Error::ChartNotTracked,
				"Sepcify a username" => Error::UserNotFound, // lol "sepcify"
				"User not found" => Error::UserNotFound,
				"Could not find scores for that user" => Error::UserNotFound,
				"No users for specified country" => Error::NoUsersFound,
				"Score not found" => Error::ScoreNotFound,
				other => Error::UnknownApiError(other.to_owned()),
			})
		}

		Ok(json)
	}

	/// Retrieves detailed metadata about the score with the given id.
	/// 
	/// # Errors
	/// - [`Error::ScoreNotFound`] if the given song id doesn't exist
	/// 
	/// # Example
	/// ```rust
	/// let song = session.song_data(2858)?;
	/// 
	/// assert_eq!(song.name, "Game Time");
	/// ```
	pub fn song_data(&mut self, song_id: u32) -> Result<SongData, Error> {
		let json = self.request("song", &[("key", song_id.to_string().as_str())])?;
		let json = json.singular_array_item()?;

		if json["songkey"].is_null() {
			return Err(Error::SongNotFound);
		}

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

	/// Retrieves an Etterna version string. I don't know what this specific version string stands
	/// for. Maybe the minimum version that the site was tested with? I don't know
	/// 
	/// # Example
	/// ```rust
	/// let client_version = session.client_version()?;
	/// assert_eq!(client_version, "0.70.1"); // As of 2020-07-25
	/// ```
	pub fn client_version(&mut self) -> Result<String, Error> {
		Ok(self.request("clientVersion", &[])?["version"].string()?)
	}

	/// Retrieve the link where you can register for an EO account
	/// 
	/// # Example
	/// ```rust
	/// let register_link = session.register_link()?;
	/// assert_eq!(register_link, "https://etternaonline.com/user/register/"); // As of 2020-07-25
	/// ```
	pub fn register_link(&mut self) -> Result<String, Error> {
		Ok(self.request("registerLink", &[])?["link"].string()?)
	}

	/// Retrieves a list of all packs tracked on EO
	/// 
	/// # Example
	/// ```rust
	/// let pack_list = session.pack_list()?;
	/// 
	/// // As of 2020-07-25
	/// assert_eq!(pack_list[0].name, "'c**t");
	/// assert_eq!(pack_list[1].name, "'d");
	/// ```
	pub fn pack_list(&mut self) -> Result<Vec<PackEntry>, Error> {
		let json = self.request("pack_list", &[])?;
		json.array()?.iter().map(|json| Ok(PackEntry {
			id: json["packid"].u32_()?,
			name: json["packname"].string()?,
			average_msd: json["average"].f32_()?,
			date_added: json["date"].string()?,
			download_link: json["download"].string()?,
			download_link_mirror: json["mirror"].string()?,
			size: FileSize::from_bytes(json["size"].u64_()?),
		})).collect()
	}

	/// Retrieves the leaderboard for a chart, which includes the replay data for each leaderboard
	/// entry
	/// 
	/// # Errors
	/// - [`Error::ChartNotTracked`] if the given chartkey is not tracked on EO
	/// 
	/// # Example
	/// ```rust
	/// let leaderboard = session.chart_leaderboard("Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d3")?;
	/// assert_eq!(leaderboard[0].user.username, "kangalioo"); // As of 2020-07-25
	/// ```
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
				country_code: json["countrycode"].string()?,
				rating: json["player_rating"].f32_string()?,
			},
			replay: crate::common::parse_replay(&json["replay"])?,
		})).collect()
	}

	/// Retrieves the user's ten latest scores
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the specified user does not exist
	/// 
	/// # Example
	/// ```rust
	/// let latest_scores = session.user_latest_10_scores("kangalioo")?;
	/// println!("Last played song was {}", latest_scores[0].song_name);
	/// ```
	pub fn user_latest_10_scores(&mut self, username: &str) -> Result<Vec<LatestScore>, Error> {
		let json = self.request("last_user_session", &[("username", username)])?;

		json.array()?.iter().map(|json| Ok(LatestScore {
			song_name: json["songname"].string()?,
			rate: json["user_chart_rate_rate"].f32_string()?,
			ssr_overall: json["Overall"].f32_string()?,
			wifescore: json["wifescore"].f32_string()?,
		})).collect()
	}

	/// Retrieves detailed data about the user
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the specified user does not exist
	/// 
	/// # Example
	/// ```rust
	/// let me = session.user_data("kangalioo")?;
	/// 
	/// assert_eq!(&me.country_code, "DE");
	/// assert_eq!(&me.is_moderator, false);
	/// ```
	pub fn user_data(&mut self, username: &str) -> Result<UserData, Error> {
		let json = self.request("user_data", &[("username", username)])?;

		Ok(UserData {
			user_name: json["username"].string()?, // "kangalioo"
			about_me: json["aboutme"].string()?, // "<p>I'm a very, very mysterious person.</p>"
			country_code: json["countrycode"].string()?, // "DE"
			is_moderator: json["moderator"].bool_int_string()?, // "0"
			avatar: json["avatar"].string()?, // "251c375b7c64494a304ea4d3a55afa92.jpg"
			default_modifiers: json["default_modifiers"].string_maybe()?, // null
			rating: UserSkillsets {
				stream: json["Stream"].f32_string()?, // "27.5298"
				jumpstream: json["Jumpstream"].f32_string()?, // "27.4409"
				handstream: json["Handstream"].f32_string()?, // "28.1328"
				stamina: json["Stamina"].f32_string()?, // "27.625"
				jackspeed: json["JackSpeed"].f32_string()?, // "25.3525"
				chordjack: json["Chordjack"].f32_string()?, // "27.479"
				technical: json["Technical"].f32_string()?, // "27.7202"
			},
			is_patreon: if json["Patreon"].is_null() { // null
				false
			} else {
				json["Patreon"].bool_int_string()?
			},
		})
	}

	/// Retrieves the user's rank for each skillset, including overall
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the specified user does not exist
	/// 
	/// # Example
	/// ```rust
	/// let ranks = session.user_ranks("kangalioo");
	/// 
	/// // As of 2020-07-25 (who knows)
	/// assert!(ranks.handstream < ranks.jackspeed);
	/// ```
	pub fn user_ranks(&mut self, username: &str) -> Result<UserRank, Error> {
		let json = self.request("user_rank", &[("username", username)])?;

		let user_rank = UserRank {
			overall: json["Overall"].u32_string()?,
			stream: json["Stream"].u32_string()?,
			jumpstream: json["Jumpstream"].u32_string()?,
			handstream: json["Handstream"].u32_string()?,
			stamina: json["Stamina"].u32_string()?,
			jackspeed: json["JackSpeed"].u32_string()?,
			chordjack: json["Chordjack"].u32_string()?,
			technical: json["Technical"].u32_string()?,
		};

		let user_rank_when_user_not_found = UserRank { overall: 1, stream: 1, jumpstream: 1,
			handstream: 1, stamina: 1, jackspeed: 1, chordjack: 1, technical: 1 };
		if user_rank == user_rank_when_user_not_found {
			return Err(Error::UserNotFound);
		}

		Ok(user_rank)
	}

	/// Retrieve the user's top scores, either overall or in a specific skillset
	/// 
	/// If the number of requested results exceeds the total number of scores, or if number is zero,
	/// all scores are returned
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the specified user does not exist
	/// 
	/// # Example
	/// ```rust
	/// let top_jumpstream_scores = session.user_top_scores(
	/// 	"kangalioo",
	/// 	Skillset8::Jumpstream,
	/// 	5,
	/// )?;
	/// 
	/// assert_eq!(&top_jumpstream_scores[0].song_name, "Everytime I hear Your Name");
	/// ```
	pub fn user_top_scores(&mut self,
		username: &str,
		skillset: Skillset8,
		number: u32
	) -> Result<Vec<TopScore>, Error> {
		let json = self.request("user_top_scores", &[
			("username", username),
			("ss", skillset.into_skillset7().map(crate::common::skillset_to_eo).unwrap_or("")),
			("num", &number.to_string()),
		])?;
		
		json.array()?.iter().map(|json| Ok(TopScore {
			song_name: json["songname"].string()?, // "Everytime I hear Your Name"
			rate: json["user_chart_rate_rate"].f32_string()?, // "1.40"
			ssr_overall: json["Overall"].f32_string()?, // "30.78"
			wifescore: json["wifescore"].f32_string()?, // "0.96986"
			chartkey: json["chartkey"].string()?, // "X4b537c03eb1f72168f51a0ab92f8a58a62fbe4b4"
			scorekey: json["scorekey"].string()?, // "S11f0f01ab55220ebbf4e0e5ee28d36cce9a72721"
			difficulty: json["difficulty"].difficulty_string()?, // "Hard"
		})).collect()
	}

	fn generic_leaderboard(&mut self,
		params: &[(&str, &str)]
	) -> Result<Vec<CountryLeaderboardEntry>, Error> {
		let json = self.request("leaderboard", params)?;

		json.array()?.iter().map(|json| Ok(CountryLeaderboardEntry {
			username: json["username"].string()?,
			avatar: json["avatar"].string()?,
			rating: user_skillsets_from_eo(json)?,
			country_code: json["countrycode"].string()?,
		})).collect()
	}

	/// Retrieves the player leaderboard for the given country.
	/// 
	/// # Errors
	/// - [`Error::NoUsersFound`] if there are no users registered in this country
	/// 
	/// # Example
	/// ```rust
	/// let leaderboard = session.country_leaderboard("DE")?
	/// 
	/// println!(
	/// 	"The best German Etterna player is {} with a rating of {}",
	/// 	leaderboard[0].user.username,
	/// 	leaderboard[0].rating.overall(),
	/// );
	/// ```
	pub fn country_leaderboard(&mut self,
		country_code: &str,
	) -> Result<Vec<CountryLeaderboardEntry>, Error> {
		self.generic_leaderboard(&[("cc", country_code)])
	}

	/// Retrieves the worldwide leaderboard of players.
	/// 
	/// # Example
	/// ```rust
	/// let leaderboard = session.world_leaderboard()?;
	/// 
	/// println!(
	/// 	"The world's best Etterna player is {} with a rating of {}",
	/// 	leaderboard[0].user.username,
	/// 	leaderboard[0].rating.overall(),
	/// );
	/// ```
	pub fn global_leaderboard(&mut self) -> Result<Vec<CountryLeaderboardEntry>, Error> {
		self.generic_leaderboard(&[])
	}

	/// Retrieves detailed metadata and the replay data about the score with the given scorekey.
	/// 
	/// # Errors
	/// - [`Error::ScoreNotFound`] if the supplied scorekey was not found
	/// 
	/// # Example
	/// ```
	/// let score_info = session.score_data("S11f0f01ab55220ebbf4e0e5ee28d36cce9a72722")?;
	/// 
	/// assert_eq!(score_info.max_combo, 1026)
	/// ```
	pub fn score_data(&mut self, scorekey: &str) -> Result<ScoreData, Error> {
		let json = self.request("score", &[("key", scorekey)])?;
		let json = json.singular_array_item()?;

		Ok(ScoreData {
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
				country_code: json["countrycode"].string()?,
				rating: json["player_rating"].f32_string()?,
			},
			replay: crate::common::parse_replay(&json["replay"])?,
			song: Song {
				name: json["songname"].string()?,
				artist: json["artist"].string()?,
				id: json["id"].u32_string()?,
			}
		})
	}
}

/*
clientVersion
registerLink
chartLeaderboard - chartkey: chartkey
song - key: songkey
last_user_session - username: username
destroy
pack_list
user_data - username: username
user_rank - username: username
user_top_scores - username: username, ss?: skillset, num?: number of scores
login - username: username, password: password
leaderboard - cc?: country code
score - key: scorekey
*/