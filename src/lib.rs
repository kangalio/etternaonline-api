#![allow(clippy::tabs_in_doc_comments)]

/*!
This crate provides an ergonomic wrapper around the v2 API of
[EtternaOnline](https://etternaonline.com), commonly abbreviated "EO". The EO API requires a valid
username and password combination to expose its functions. You will also need an API token called
"client data".

# Usage
For detailed documentation usage, see [`Session`]
*/

// THIS IS MY TODO LIST:
// - Remove thiserror dependency

mod structs;
pub use structs::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("User not found")]
	UserNotFound,
	#[error("Username and password combination not found")]
	InvalidLogin,
	#[error("Server response was malformed or unsensical")]
	UnexpectedResponse(String),
	#[error("Error while parsing the json sent by the server")]
	InvalidJson(String),
	#[error("Score not found")]
	ScoreNotFound,
	#[error("Chart not tracked")]
	ChartNotTracked,
	#[error("Favorite already exists")]
	ChartAlreadyFavorited,
	#[error("Database error")]
	DatabaseError,
	#[error("Goal already exists")]
	GoalAlreadyExists,
	#[error("Chart already exists")]
	ChartAlreadyAdded,
	#[error("The uploaded file is not a valid XML file")]
	InvalidXml,
	#[error("No users registered")]
	NoUsersFound,
	#[error("Web server is down")]
	ServerIsDown,
	#[error("An unknown EO API error")]
	UnknownApiError(String),
	#[error("Server timed out")]
	Timeout,
}

fn difficulty_from_eo(string: &str) -> Result<Difficulty, Error> {
	Ok(match string {
		"Beginner" => Difficulty::Beginner,
		"Easy" => Difficulty::Easy,
		"Medium" => Difficulty::Medium,
		"Hard" => Difficulty::Hard,
		"Challenge" => Difficulty::Challenge,
		"Edit" => Difficulty::Edit,
		other => return Err(Error::UnexpectedResponse(format!("Unexpected difficulty name '{}'", other))),
	})
}

fn skillset7_to_eo(skillset: Skillset7) -> &'static str {
	match skillset {
		Skillset7::Stream => "Stream",
		Skillset7::Jumpstream => "Jumpstream",
		Skillset7::Handstream => "Handstream",
		Skillset7::Stamina => "Stamina",
		Skillset7::Jackspeed => "JackSpeed",
		Skillset7::Chordjack => "Chordjack",
		Skillset7::Technical => "Technical",
	}
}

fn skillsets_from_eo(json: &serde_json::Value) -> Skillsets {
	Skillsets {
		stream: json["Stream"].as_f64().unwrap(),
		jumpstream: json["Jumpstream"].as_f64().unwrap(),
		handstream: json["Handstream"].as_f64().unwrap(),
		stamina: json["Stamina"].as_f64().unwrap(),
		jackspeed: json["JackSpeed"].as_f64().unwrap(),
		chordjack: json["Chordjack"].as_f64().unwrap(),
		technical: json["Technical"].as_f64().unwrap(),
	}
}

fn note_type_from_eo(note_type: &serde_json::Value) -> Result<NoteType, Error> {
	match note_type.as_i64().unwrap() {
		1 => Ok(NoteType::Tap),
		2 => Ok(NoteType::HoldHead),
		3 => Ok(NoteType::HoldTail),
		4 => Ok(NoteType::Mine),
		5 => Ok(NoteType::Lift),
		6 => Ok(NoteType::Keysound),
		7 => Ok(NoteType::Fake),
		other => Err(Error::UnexpectedResponse(format!("Unexpected note type integer {}", other))),
	}
}

fn parse_judgements(json: &serde_json::Value) -> Judgements {
	Judgements {
		marvelouses: json["marvelous"].as_i64().unwrap() as u32,
		perfects: json["perfect"].as_i64().unwrap() as u32,
		greats: json["great"].as_i64().unwrap() as u32,
		goods: json["good"].as_i64().unwrap() as u32,
		bads: json["bad"].as_i64().unwrap() as u32,
		misss: json["miss"].as_i64().unwrap() as u32,
		hit_mines: json["hitMines"].as_i64().unwrap() as u32,
		held_holds: json["heldHold"].as_i64().unwrap() as u32,
		let_go_holds: json["letGoHold"].as_i64().unwrap() as u32,
		missed_holds: json["missedHold"].as_i64().unwrap() as u32,
	}
}

fn parse_replay(json: &serde_json::Value) -> Result<Option<Replay>, Error> {
	let replay_str = match json.as_array().unwrap()[0].as_str() {
		Some(replay_str) => replay_str,
		None => return Ok(None),
	};

	let json: serde_json::Value = serde_json::from_str(replay_str)
		.map_err(|e| Error::InvalidJson(format!("{}", e)))?;

	let mut notes = Vec::new();
	for note_json in json.as_array().unwrap() {
		let note_json = note_json.as_array().unwrap();
		notes.push(ReplayNote {
			time: note_json[0].as_f64().unwrap(),
			deviation: note_json[1].as_f64().unwrap() / 1000.0,
			lane: note_json[2].as_i64().unwrap() as u8,
			note_type: note_type_from_eo(&note_json[3])?,
			tick: note_json[4].as_i64().unwrap() as u32,
		});
	}

	Ok(Some(Replay { notes }))
}

fn parse_score_data_user_1(json: &serde_json::Value) -> ScoreUser {
	ScoreUser {
		username: json["userName"].as_str().unwrap().to_owned(),
		avatar: json["avatar"].as_str().unwrap().to_owned(),
		country_code: json["countryCode"].as_str().unwrap().to_owned(),
		overall_rating: json["playerRating"].as_f64().unwrap(),
	}
}

fn parse_score_data_user_2(json: &serde_json::Value) -> ScoreUser {
	ScoreUser {
		username: json["username"].as_str().unwrap().to_owned(),
		avatar: json["avatar"].as_str().unwrap().to_owned(),
		country_code: json["countryCode"].as_str().unwrap().to_owned(),
		overall_rating: json["Overall"].as_f64().unwrap(),
	}
}

/// EtternaOnline API session client, handles all requests to and from EtternaOnline.
/// 
/// This wrapper keeps cares of expiring tokens by automatically logging back in when the login
/// token expires.
/// 
/// This session has rate-limiting built-in. Please do make use of it - the EO server is brittle and
/// funded entirely by donations.
/// 
/// Initialize a session using [`Session::new_from_login`]
/// 
/// # Notes
/// Etterna terminology:
/// - The calculated difficulty for a chart is called MSD: Mina standardized difficulty.
/// - The score rating - which is variable depending on your wifescore - is called SSR:
///   score-specific-rating
/// 
/// The wifescores in this library are scaled to a maximum of `1.0`. This is means that a wifescore
/// of 100% corresponds to a value of `1.0` (as opposed to `100.0`).
/// 
/// # Example
/// ```rust
/// let mut session = Session::new_from_login(
/// 	"kangalioo",
/// 	"<PASSWORD>",
/// 	"<CLIENT_DATA>",
/// 	std::time::Duration::from_millis(2000), // Wait 2s inbetween requests
/// );
/// 
/// println!("Details about kangalioo: {:?}", session.user_details("kangalioo")?);
/// 
/// let best_score = session.user_top_10_scores("kangalioo")?[0];
/// println!(
/// 	"kangalioo's best score has {} misses",
/// 	session.score_data(best_score)?.judgements.misses
/// );
/// ```
pub struct Session {
	// This stuff is needed for re-login
	username: String,
	password: String,
	client_data: String,

	// The auth key that we get from the server on login
	authorization: String,
	
	// Rate limiting stuff
	last_request: std::time::Instant,
	rate_limit: std::time::Duration,

	timeout: Option<std::time::Duration>,
}

impl Session {
	/// Initiate a new session by logging in using the specified credentials and API token.
	/// 
	/// Rate-limiting is done by waiting at least `rate_limit` inbetween requests
	/// 
	/// # Errors
	/// - [`Error::InvalidLogin`] if username or password are wrong
	/// 
	/// # Example
	/// ```rust
	/// let mut session = Session::new_from_login(
	/// 	"kangalioo",
	/// 	"<PASSWORD>",
	/// 	"<CLIENT_DATA>",
	/// 	std::time::Duration::from_millis(2000), // wait 2s inbetween requests
	/// 	None, // no timeout
	/// );
	/// 
	/// println!("Details about kangalioo: {:?}", session.user_details("kangalioo"));
	/// ```
	pub fn new_from_login(
		username: String,
		password: String,
		client_data: String,
		rate_limit: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Result<Self, Error> {
		let authorization = "dummy key that will be replaced anyway when I login".into();

		let mut session = Self {
			username, password, client_data, authorization, rate_limit, timeout,
			last_request: std::time::Instant::now(),
		};
		session.login()?;

		Ok(session)
	}

	// login again to generate a new session token
	// hmmm I wonder if there's a risk that the server won't properly generate a session token,
	// return Unauthorized, and then my client will try to login to get a fresh token, and the
	// process repeats indefinitely...? I just hope that the EO server never throws an Unauthorized
	// on login
	fn login(&mut self) -> Result<u64, Error> {
		let form: &[(&str, &str)] = &[
			// eh fuck it. I dont wanna bother with those lifetime headaches
			// who needs allocation efficiency anyways
			("username", &self.username.clone()),
			("password", &self.password.clone()),
			("clientData", &self.client_data.clone()),
		];

		let json = self.generic_request(
			"POST",
			"login",
			|mut request| request.send_form(form),
			false,
			false,
		)?;

		self.authorization = format!(
			"Bearer {}",
			json["attributes"]["accessToken"].as_str().unwrap(),
		);

		Ok(json["attributes"]["expiresAt"].as_i64().unwrap() as u64)
	}

	fn generic_request(&mut self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response,
		do_authorization: bool,
		do_rate_limiting: bool,
	) -> Result<serde_json::Value, Error> {

		if do_rate_limiting {
			let now = std::time::Instant::now();
			let time_since_last_request = now.duration_since(self.last_request);
			if time_since_last_request < self.rate_limit {
				std::thread::sleep(self.rate_limit - time_since_last_request);
			}
			self.last_request = now;
		}

		let mut request = ureq::request(
			method,
			&format!("https://api.etternaonline.com/v2/{}", path)
		);
		if let Some(timeout) = self.timeout {
			request.timeout(timeout);
		}
		if do_authorization {
			request.set("Authorization", &self.authorization);
		}

		let response = request_callback(request);
		
		if let Some(ureq::Error::Io(io_err)) = response.synthetic_error() {
			if io_err.kind() == std::io::ErrorKind::TimedOut {
				return Err(Error::Timeout);
			}
		}

		let status = response.status();
		let mut json = match response.into_json() {
			Ok(json) => json,
			Err(e) => {
				let error_msg = format!("{}", e);
				if error_msg.contains("timed out reading response") {
					// yes, there are two places where timeouts can happen :p
					// see https://github.com/algesten/ureq/issues/119
					return Err(Error::Timeout);
				} else {
					return Err(Error::InvalidJson(error_msg));
				}
			}
		};
		
		// Error handling
		if status >= 400 {
			// BAHAHAHAHAHA I just got these three response codes _right in a row_ xDD
			// > Hit run -> "Unexpected 521" -> Add 521 special case -> hit run -> "Unexpected 503"
			// > -> Add 503 special case -> hit run -> "Unexpected 525" -> Add 525 special case
			if status == 521 || status == 503 || status == 525 {
				return Err(Error::ServerIsDown);
			}

			return match json["errors"][0]["title"].as_str().unwrap() {
				"Unauthorized" => {
					// Token expired, let's login again and retry
					self.login()?;
					return self.request(method, path, request_callback);
				},
				"Score not found" => Err(Error::ScoreNotFound),
				"Chart not tracked" => Err(Error::ChartNotTracked),
				"User not found" => Err(Error::UserNotFound),
				"Favorite already exists" => Err(Error::ChartAlreadyFavorited),
				"Database error" => Err(Error::DatabaseError),
				"Goal already exist" => Err(Error::GoalAlreadyExists),
				"Chart already exists" => Err(Error::ChartAlreadyAdded),
				"Malformed XML file" => Err(Error::InvalidXml),
				"No users found" => Err(Error::NoUsersFound),
				other => Err(Error::UnknownApiError(other.to_owned())),
			};
		} else if status != 200 {
			// TODO: should we have print calls in a library?
			println!("Warning: status code {}", status);
		}

		Ok(json["data"].take())
	}

	fn request(&mut self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response
	) -> Result<serde_json::Value, Error> {
		self.generic_request(method, path, request_callback, true, true)
	}

	fn get(&mut self, path: &str) -> Result<serde_json::Value, Error> {
		self.request("GET", path, |mut request| request.call())
	}

	/// Retrieves details about the profile of the specified user.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```
	/// // Retrieve details about user "kangalioo"
	/// let details = session.user_details("kangalioo")?;
	/// ```
	pub fn user_details(&mut self, username: &str) -> Result<UserDetails, Error> {
		let json = self.get(&format!("user/{}", username))?;
		let json = &json["attributes"];

		Ok(UserDetails {
			username: json["userName"].as_str().unwrap().to_owned(),
			about_me: json["aboutMe"].as_str().unwrap().to_owned(),
			is_moderator: json["moderator"].as_bool().unwrap(),
			is_patreon: json["patreon"].as_bool().unwrap(),
			avatar_url: json["avatar"].as_str().unwrap().to_owned(),
			country_code: json["countryCode"].as_str().unwrap().to_owned(),
			player_rating: json["playerRating"].as_f64().unwrap(),
			default_modifiers: json["defaultModifiers"].as_str().unwrap().to_owned(),
			rating: skillsets_from_eo(&json["skillsets"]),
		})
	}
	
	fn parse_top_scores(&mut self, url: &str) -> Result<Vec<TopScore>, Error> {
		let json = self.get(url)?;

		let mut scores = Vec::new();
		for score_json in json.as_array().unwrap() {
			let difficulty = difficulty_from_eo(score_json["attributes"]["difficulty"].as_str().unwrap())?;

			// println!("{:#?}", json);
			scores.push(TopScore {
				scorekey: score_json["id"].as_str().unwrap().to_owned(),
				song_name: score_json["attributes"]["songName"].as_str().unwrap().to_owned(),
				ssr_overall: score_json["attributes"]["Overall"].as_f64().unwrap(),
				wifescore: score_json["attributes"]["wife"].as_f64().unwrap() / 100.0,
				rate: score_json["attributes"]["rate"].as_f64().unwrap(),
				difficulty,
				chartkey: score_json["attributes"]["chartKey"].as_str().unwrap().to_owned(),
				base_msd: skillsets_from_eo(&score_json["attributes"]["skillsets"]),
			});
		}

		Ok(scores)
	}

	/// Retrieve the user's top scores by the given skillset. The number of scores returned is equal
	/// to `limit`
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```
	/// // Retrieve the top 10 chordjack scores of user "kangalioo"
	/// let scores = session.user_top_skillset_scores("kangalioo", Skillset7::Chordjack, 10)?;
	/// ```
	pub fn user_top_skillset_scores(&mut self,
		username: &str,
		skillset: Skillset7,
		limit: u32,
	) -> Result<Vec<TopScore>, Error> {
		self.parse_top_scores(&format!(
			"user/{}/top/{}/{}",
			username, skillset7_to_eo(skillset), limit
		))
	}

	/// Retrieve the user's top 10 scores, sorted by the overall SSR. Due to a bug in the EO v2 API,
	/// it's unfortunately not possible to control the number of scores returned.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```
	/// // Retrieve the top 10 scores of user "kangalioo"
	/// let scores = session.user_top_10_scores("kangalioo")?;
	/// ```
	pub fn user_top_10_scores(&mut self, username: &str) -> Result<Vec<TopScore>, Error> {
		self.parse_top_scores(&format!("user/{}/top//", username))
	}
	
	/// Retrieve the user's latest 10 scores.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```
	/// // Retrieve the latest 10 scores of user "kangalioo"
	/// let scores = session.user_latest_scores("kangalioo")?;
	/// ```
	pub fn user_latest_scores(&mut self, username: &str) -> Result<Vec<LatestScore>, Error> {
		let json = self.get(&format!("user/{}/latest", username))?;

		let mut scores = Vec::new();
		for score_json in json.as_array().unwrap() {
			scores.push(LatestScore {
				scorekey: score_json["id"].as_str().unwrap().to_owned(),
				song_name: score_json["attributes"]["songName"].as_str().unwrap().to_owned(),
				ssr_overall: score_json["attributes"]["Overall"].as_f64().unwrap(),
				wifescore: score_json["attributes"]["wife"].as_f64().unwrap() / 100.0,
				rate: score_json["attributes"]["rate"].as_f64().unwrap(),
				difficulty: difficulty_from_eo(score_json["attributes"]["difficulty"].as_str().unwrap())?,
			});
		}

		Ok(scores)
	}

	/// Retrieve the user's rank for each skillset.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```
	/// // Retrieve "kangalioo"'s rank for each skillset
	/// let scores = session.user_ranks("kangalioo")?;
	/// ```
	pub fn user_ranks_per_skillset(&mut self, username: &str) -> Result<UserRanksPerSkillset, Error> {
		let json = self.get(&format!("user/{}/ranks", username))?;
		let json = &json["attributes"];

		Ok(UserRanksPerSkillset {
			overall: json["Overall"].as_i64().unwrap() as u32,
			stream: json["Stream"].as_i64().unwrap() as u32,
			jumpstream: json["Jumpstream"].as_i64().unwrap() as u32,
			handstream: json["Handstream"].as_i64().unwrap() as u32,
			stamina: json["Stamina"].as_i64().unwrap() as u32,
			jackspeed: json["JackSpeed"].as_i64().unwrap() as u32,
			chordjack: json["Chordjack"].as_i64().unwrap() as u32,
			technical: json["Technical"].as_i64().unwrap() as u32,
		})
	}

	/// Retrieve the user's best scores for each skillset. The number of scores yielded is not
	/// documented in the EO API, but according to my experiments it's 25.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```rust
	/// let top_scores = session.user_top_scores_per_skillset("kangalioo")?;
	/// println!("kangalioo's 5th best handstream score is {:?}", top_scores.handstream[4]);
	/// ```
	pub fn user_top_scores_per_skillset(&mut self,
		username: &str,
	) -> Result<UserTopScoresPerSkillset, Error> {
		let json = self.get(&format!("user/{}/all", username))?;

		let parse_skillset_top_scores = |array: &serde_json::Value| {
			let mut scores = Vec::new();
			for score_json in array.as_array().unwrap() {
				scores.push(TopScorePerSkillset {
					song_name: score_json["songname"].as_str().unwrap().to_owned(),
					rate: score_json["user_chart_rate_rate"].as_f64().unwrap(),
					wifescore: score_json["wifescore"].as_f64().unwrap(),
					chartkey: score_json["chartkey"].as_str().unwrap().to_owned(),
					scorekey: score_json["scorekey"].as_str().unwrap().to_owned(),
					difficulty: difficulty_from_eo(score_json["difficulty"].as_str().unwrap())?,
					ssr: skillsets_from_eo(&score_json),
				})
			}

			Ok(scores)
		};

		Ok(UserTopScoresPerSkillset {
			overall: parse_skillset_top_scores(&json["attributes"]["Overall"])?,
			stream: parse_skillset_top_scores(&json["attributes"]["Stream"])?,
			jumpstream: parse_skillset_top_scores(&json["attributes"]["Jumpstream"])?,
			handstream: parse_skillset_top_scores(&json["attributes"]["Handstream"])?,
			stamina: parse_skillset_top_scores(&json["attributes"]["Stamina"])?,
			jackspeed: parse_skillset_top_scores(&json["attributes"]["JackSpeed"])?,
			chordjack: parse_skillset_top_scores(&json["attributes"]["Chordjack"])?,
			technical: parse_skillset_top_scores(&json["attributes"]["Technical"])?,
		})
	}

	/// Retrieves detailed metadata and the replay data about the score with the given scorekey.
	/// 
	/// # Errors
	/// - [`Error::ScoreNotFound`] if the supplied scorekey was not found
	/// 
	/// # Example
	/// ```
	/// let score_info = session.score_data("S65565b5bc377c6d78b60c0aecfd9e05955b4cf63")?;
	/// ```
	pub fn score_data(&mut self, scorekey: &str) -> Result<ScoreData, Error> {
		let json = self.get(&format!("score/{}", scorekey))?;

		let scorekey = json["id"].as_str().unwrap().to_owned();
		let json = &json["attributes"];

		Ok(ScoreData {
			scorekey,
			wifescore: json["wife"].as_f64().unwrap(),
			rate: json["rate"].as_f64().unwrap(),
			max_combo: json["maxCombo"].as_i64().unwrap() as u32,
			is_valid: json["valid"].as_bool().unwrap(),
			has_chord_cohesion: !json["nocc"].as_bool().unwrap(),
			song_name: json["song"]["songName"].as_str().unwrap().to_owned(),
			artist: json["song"]["artist"].as_str().unwrap().to_owned(),
			song_id: json["song"]["id"].as_i64().unwrap() as u32,
			ssr: skillsets_from_eo(&json["skillsets"]),
			judgements: parse_judgements(&json["judgements"]),
			replay: parse_replay(&json["replay"])?,
			user: parse_score_data_user_2(&json["user"]),
		})
	}

	/// Retrieves the leaderboard for the specified chart. The return type is a vector of
	/// leaderboard entries.
	/// 
	/// # Errors
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	/// 
	/// # Example
	/// ```rust
	/// let leaderboard = session.chart_leaderboard("X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	/// 
	/// println!("The best Game Time score is being held by {}", leaderboard[0].user.username);
	/// ```
	pub fn chart_leaderboard(&mut self, chartkey: &str) -> Result<Vec<ChartLeaderboardScore>, Error> {
		let json = self.get(&format!("charts/{}/leaderboards", chartkey))?;

		let mut scores = Vec::new();
		for json in json.as_array().unwrap() {
			let scorekey = json["id"].as_str().unwrap().to_owned();
			let json = &json["attributes"];

			scores.push(ChartLeaderboardScore {
				scorekey,
				wifescore: json["wife"].as_f64().unwrap() / 100.0,
				max_combo: json["maxCombo"].as_i64().unwrap() as u32,
				is_valid: json["valid"].as_bool().unwrap(),
				modifiers: json["modifiers"].as_str().unwrap().to_owned(),
				has_chord_cohesion: !json["noCC"].as_bool().unwrap(),
				rate: json["rate"].as_f64().unwrap(),
				datetime: json["datetime"].as_str().unwrap().to_owned(),
				ssr: skillsets_from_eo(&json["skillsets"]),
				judgements: parse_judgements(&json["judgements"]),
				has_replay: json["hasReplay"].as_bool().unwrap(), // API docs are wrong again
				user: parse_score_data_user_1(&json["user"]),
			});
		}

		Ok(scores)
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
	pub fn country_leaderboard(&mut self, country_code: &str) -> Result<Vec<LeaderboardEntry>, Error> {
		let json = self.get(&format!("leaderboard/{}", country_code))?;

		let mut entries = Vec::new();
		for json in json.as_array().unwrap() {
			entries.push(LeaderboardEntry {
				user: parse_score_data_user_2(&json["attributes"]["user"]),
				rating: skillsets_from_eo(&json["attributes"]["skillsets"]),
			});
		}

		Ok(entries)
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
	pub fn world_leaderboard(&mut self) -> Result<Vec<LeaderboardEntry>, Error> {
		self.country_leaderboard("")
	}

	/// Retrieves the user's favorites. Returns a vector of chartkeys.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	/// 
	/// # Example
	/// ```rust
	/// let favorites = session.user_favorites("kangalioo");
	/// println!("kangalioo has {} favorites", favorites.len());
	/// ```
	pub fn user_favorites(&mut self, username: &str) -> Result<Vec<String>, Error> {
		let json = self.get(&format!("user/{}/favorites", username))?;

		let chartkeys: Vec<String> = json.as_array().unwrap().iter()
			.map(|obj| obj["attributes"]["chartkey"].as_str().unwrap().to_owned())
			.collect();

		Ok(chartkeys)
	}

	/// Add a chart to the user's favorites.
	/// 
	/// # Errors
	/// - [`Error::ChartAlreadyFavorited`] if the chart is already in the user's favorites
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	/// 
	/// # Example
	/// ```rust
	/// // Favorite Game Time
	/// session.add_user_favorite("kangalioo", "X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	/// ```
	pub fn add_user_favorite(&mut self, username: &str, chartkey: &str) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/favorites", username),
			|mut req| req.send_form(&[("chartkey", chartkey)]),
		)?;

		Ok(())
	}

	/// Remove a chart from the user's favorites.
	/// 
	/// # Example
	/// ```rust
	/// // Unfavorite Game Time
	/// session.remove_user_favorite("kangalioo", "X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	/// ```
	pub fn remove_user_favorite(&mut self, username: &str, chartkey: &str) -> Result<(), Error> {
		self.request(
			"DELETE",
			&format!("user/{}/favorites/{}", username, chartkey),
			|mut request| request.call()
		)?;

		Ok(())
	}

	/// Retrieves a user's score goals.
	/// 
	/// # Errors
	/// - [`Error::UserNotFound`] if the specified user doesn't exist or if the specified user has no
	///   goals
	/// 
	/// # Example
	/// ```rust
	/// let score_goals = session.user_goals("theropfather")?;
	/// 
	/// println!("theropfather has {} goals", score_goals.len());
	/// ```
	pub fn user_goals(&mut self, username: &str) -> Result<Vec<ScoreGoal>, Error> {
		let json = self.get(&format!("user/{}/goals", username))?;

		let score_goals: Vec<ScoreGoal> = json.as_array().unwrap().iter()
			.map(|json| ScoreGoal {
				chartkey: json["attributes"]["chartkey"].as_str().unwrap().to_owned(),
				rate: json["attributes"]["rate"].as_f64().unwrap(),
				wifescore: json["attributes"]["wife"].as_f64().unwrap(),
				time_assigned: json["attributes"]["timeAssigned"].as_str().unwrap().to_owned(),
				time_achieved: if json["attributes"]["achieved"].as_i64().unwrap() == 0 {
					None
				} else {
					Some(json["attributes"]["timeAchieved"].as_str().unwrap().to_owned())
				}
			})
			.collect();

		Ok(score_goals)
	}

	/// Add a new score goal.
	/// 
	/// # Errors
	/// - [`Error::GoalAlreadyExists`] when the goal already exists in the database
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	/// - [`Error::DatabaseError`] if there was a problem with the database
	/// 
	/// # Example
	/// ```rust
	/// // Add a Game Time 1.0x AA score goal
	/// session.add_user_goal(
	/// 	"kangalioo",
	/// 	"X4a15f62b66a80b62ec64521704f98c6c03d98e03",
	/// 	1.0,
	/// 	0.93,
	/// 	"2020-07-13 22:48:26",
	/// )?;
	/// ```
	// TODO: somehow enforce that `time_assigned` is valid ISO 8601
	pub fn add_user_goal(&mut self,
		username: &str,
		chartkey: &str,
		rate: f64,
		wifescore: f64,
		time_assigned: &str,
	) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/goals", username),
			|mut request| request.send_form(&[
				("chartkey", chartkey),
				("rate", &format!("{}", rate)),
				("wife", &format!("{}", wifescore)),
				("timeAssigned", time_assigned),
			])
		)?;

		Ok(())
	}

	/// Remove the user goal with the specified chartkey, rate and wifescore.
	/// 
	/// Note: this API call doesn't seem to do anything
	/// 
	/// # Example
	/// ```rust
	/// // Let's delete theropfather's first score goal
	/// 
	/// let score_goal = session.user_goals("theropfather")?[0];
	/// 
	/// session.remove_user_goal(
	/// 	"theropfather",
	/// 	score_goal.chartkey,
	/// 	score_goal.rate,
	/// 	score_goal.wifescore
	/// )?;
	/// ```
	pub fn remove_user_goal(&mut self,
		username: &str,
		chartkey: &str,
		rate: f64,
		wifescore: f64,
	) -> Result<(), Error> {
		let rate = format!("{}", rate);
		let wifescore = format!("{}", wifescore);
		self.request(
			"DELETE",
			&format!("user/{}/goals/{}/{}/{}", username, chartkey, wifescore, rate),
			|mut request| request.call(),
		)?;

		Ok(())
	}

	/// Update a score goal by replacing all its attributes with the given ones.
	/// 
	/// # Example
	/// ```rust
	/// // Let's up kangalioo's first score goal's rate by 0.05
	/// 
	/// let score_goal = &mut session.user_goals("kangalioo")?[0];
	/// 
	/// score_goal.rate += 0.05;
	/// 
	/// session.update_user_goal("kangalioo", score_goal)?;
	/// ```
	pub fn update_user_goal(&mut self, username: &str, goal: &ScoreGoal) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/goals/update", username),
			|mut request| request.send_form(&[
				("chartkey", &goal.chartkey),
				("timeAssigned", &goal.time_assigned),
				("achieved", if goal.time_achieved.is_some() { "1" } else { "0" }),
				("rate", &format!("{}", goal.rate)),
				("wife", &format!("{}", goal.wifescore)),
				("timeAchieved", goal.time_achieved.as_deref().unwrap_or("0000-00-00 00:00:00")),
			]),
		)?;

		Ok(())
	}

	// pub fn test(&mut self) -> Result<(), Error> {
		// let best_score = &self.user_top_10_scores("kangalioo")?[0];

		// println!("{:#?}", self.user_top_skillset_scores("kangalioo", Skillset7::Technical, 3)?);
		// println!("{:#?}", self.user_top_10_scores("kangalioo")?);
		// println!("{:#?}", self.user_details("kangalioo")?);
		// println!("{:#?}", self.user_latest_scores("kangalioo")?);
		// println!("{:#?}", self.user_ranks_per_skillset("kangalioo")?);
		// println!("{:#?}", self.user_top_scores_per_skillset("kangalioo")?);
		// println!("{:#?}", self.score_data(&best_score.scorekey));
		// println!("{:#?}", self.chart_leaderboards("Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2"));
		// println!("{:#?}", self.country_leaderboard("DE"));
		// println!("{:#?}", self.add_user_favorite("kangalioo", "Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2"));
		// println!("{:#?}", self.remove_user_favorite("kangalioo", "Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2"));
		// println!("{:#?}", self.add_user_goal("kangalioo", "Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2", 0.75, 0.8686, "2037-06-04 15:00:00"));
		// println!("{:#?}", self.remove_user_goal("kangalioo", "Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2", 0.7, 0.8686));
		// let goal = &mut self.user_goals("kangalioo")?[0];
		// goal.wifescore += 0.01;
		// println!("{:#?}", self.update_user_goal("kangalioo", &goal));
		// println!("{:#?}", self.user_goals("kangalioo")?);


		// check if wifescores are all normalized to a max of 1.0
		// println!("{} {} {} {} {} {}",
		// 	self.user_top_skillset_scores("kangalioo", Skillset7::Stream, 3)?[0].wifescore,
		// 	self.user_top_10_scores("kangalioo")?[0].wifescore,
		// 	self.user_latest_scores("kangalioo")?[0].wifescore,
		// 	self.user_top_scores_per_skillset("kangalioo")?.jackspeed[0].wifescore,
		// 	self.score_data(&best_score.scorekey)?.wifescore,
		// 	self.chart_leaderboards("Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2")?[0].wifescore,
		// );
		
		// Ok(())
	// }
}