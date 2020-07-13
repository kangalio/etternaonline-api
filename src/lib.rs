#![allow(clippy::tabs_in_doc_comments)]

/*!
This crate provides an ergonomic wrapper around the v2 API of
[EtternaOnline](https://etternaonline.com) (commonly abbreviated "EO"). The EO API requires a valid
username and password combination to expose its functions. You will also need an API token called
"client data".

# Usage
For information on usage, see [`Session`]

<!-- EXAMPLE HERE, 30 LOC OR SO -->
*/

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
	#[error("No users found in this country")]
	NoUsersInCountry,
	#[error("An unknown EO API error")]
	UnknownApiError(String),
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

fn skillsets7_from_eo(json: &serde_json::Value) -> Skillsets7 {
	Skillsets7 {
		stream: json["Stream"].as_f64().unwrap(),
		jumpstream: json["Jumpstream"].as_f64().unwrap(),
		handstream: json["Handstream"].as_f64().unwrap(),
		stamina: json["Stamina"].as_f64().unwrap(),
		jackspeed: json["JackSpeed"].as_f64().unwrap(),
		chordjack: json["Chordjack"].as_f64().unwrap(),
		technical: json["Technical"].as_f64().unwrap(),
	}
}

fn skillsets8_from_eo(json: &serde_json::Value) -> Skillsets8 {
	Skillsets8 {
		overall: json["Overall"].as_f64().unwrap(),
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
/// # Conventions
/// Etterna terminology:
/// - The calculated difficulty for a chart is called MSD: Mina standardized difficulty.
/// - The score rating - which is variable depending on your wifescore - is called SSR:
///   score-specific-rating
/// 
/// The wifescores in this library are scaled to a maximum of `1.0`. This is means that a wifescore
/// of 100% corresponds to a value of `1.0` (as opposed to `100.0`).
/// 
/// Skillset data comes in two flavors: with overall rating and without overall rating. Depending on
/// which API function you call, you get one or the either. To avoid ambiguity, there's:
/// - Without overall: [`Skillsets7`] and [`Skillset7`] 
/// - With overall: [`Skillsets8`] and [`Skillset8`]
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
}

impl Session {
	/// Initiate a new session by logging in using the specified credentials and API token.
	/// 
	/// Rate-limiting is done by waiting at least `rate_limit` inbetween requests
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
	/// println!("Details about kangalioo: {:?}", session.user_details("kangalioo"));
	/// ```
	pub fn new_from_login(
		username: String,
		password: String,
		client_data: String,
		rate_limit: std::time::Duration,
	) -> Result<Self, Error> {
		let authorization = "dummy key that will be replaced anyway when I login".into();

		let mut session = Self {
			username, password, client_data, authorization, rate_limit,
			last_request: std::time::Instant::now(),
		};
		session.login()?;

		Ok(session)
	}

	// login again to generate a new session token
	fn login(&mut self) -> Result<(), Error> {
		let response = ureq::post("https://api.etternaonline.com/v2/login")
			.send_form(&[
				("username", &self.username),
				("password", &self.password),
				("clientData", &self.client_data)
			]);

		match response.status() {
			404 => Err(Error::InvalidLogin),
			200 => {
				let json = response.into_json()
					.map_err(|e| Error::InvalidJson(format!("{}", e)))?;
				let key = json["data"]["attributes"]["accessToken"].as_str()
					.expect("Received an access token that is not a string");
				self.authorization = format!("Bearer {}", key);
				Ok(())
			}
			other => panic!("Unexpected response code {}", other),
		}
	}

	fn request(&mut self,
		builder: impl Fn() -> ureq::Request
	) -> Result<serde_json::Value, Error> {

		// Do tha rate-limiting
		let time_since_last_request = std::time::Instant::now().duration_since(self.last_request);
		if time_since_last_request < self.rate_limit {
			std::thread::sleep(self.rate_limit - time_since_last_request);
		}
		self.last_request = std::time::Instant::now();

		let response = builder()
			.set("Authorization", &self.authorization)
			.call();
		
		let status = response.status();
		let mut json = response.into_json()
			.map_err(|e| Error::InvalidJson(format!("{}", e)))?;
		
		// Error handling
		if status >= 400 {
			return match json["errors"][0]["title"].as_str().unwrap() {
				"Unauthorized" => {
					// Token expired, let's login again and retry
					self.login()?;
					return self.request(builder);
				},
				"Score not found" => Err(Error::ScoreNotFound),
				"Chart not tracked" => Err(Error::ChartNotTracked),
				"User not found" => Err(Error::UserNotFound),
				"Favorite already exists" => Err(Error::ChartAlreadyFavorited),
				"Database error" => Err(Error::DatabaseError),
				"Goal already exist" => Err(Error::GoalAlreadyExists),
				"Chart already exists" => Err(Error::ChartAlreadyAdded),
				"Malformed XML file" => Err(Error::InvalidXml),
				"No users found" => Err(Error::NoUsersInCountry),
				other => Err(Error::UnknownApiError(other.to_owned())),
			};
		}

		Ok(json["data"].take())
	}

	/// Retrieves details about the profile of the specified user.
	/// 
	/// If there is no user with the specified username, `Err(Error::UserNotFound)` is returned.
	/// 
	/// # Example
	/// ```
	/// // Retrieve details about user "kangalioo"
	/// let details = session.user_details("kangalioo")?;
	/// ```
	pub fn user_details(&mut self, username: &str) -> Result<UserDetails, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}", username)
		))?;
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
			rating: skillsets7_from_eo(&json["skillsets"]),
		})
	}
	
	fn parse_top_scores(&mut self, url: &str) -> Result<Vec<TopScore>, Error> {
		let json = self.request(|| ureq::get(url))?;

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
				ssr: skillsets7_from_eo(&score_json["attributes"]["skillsets"]),
			});
		}

		Ok(scores)
	}

	/// Retrieve the user's top scores by the given skillset. The number of scores returned is equal
	/// to `limit`
	/// 
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
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
			"https://api.etternaonline.com/v2/user/{}/top/{}/{}",
			username, skillset7_to_eo(skillset), limit
		))
	}

	/// Retrieve the user's top 10 scores, sorted by the overall SSR.
	/// 
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
	/// 
	/// # Example
	/// ```
	/// // Retrieve the top 10 scores of user "kangalioo"
	/// let scores = session.user_top_10_scores("kangalioo")?;
	/// ```
	pub fn user_top_10_scores(&mut self, username: &str) -> Result<Vec<TopScore>, Error> {
		self.parse_top_scores(&format!("https://api.etternaonline.com/v2/user/{}/top//", username))
	}
	
	/// Retrieve the user's latest 10 scores.
	/// 
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
	/// 
	/// # Example
	/// ```
	/// // Retrieve the latest 10 scores of user "kangalioo"
	/// let scores = session.user_latest_scores("kangalioo")?;
	/// ```
	pub fn user_latest_scores(&mut self, username: &str) -> Result<Vec<LatestScore>, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}/latest", username)
		))?;

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
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
	/// 
	/// # Example
	/// ```
	/// // Retrieve "kangalioo"'s rank for each skillset
	/// let scores = session.user_ranks("kangalioo")?;
	/// ```
	pub fn user_ranks_per_skillset(&mut self, username: &str) -> Result<UserRanksPerSkillset, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}/ranks", username)
		))?;
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
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
	/// 
	/// # Example
	/// ```rust
	/// let top_scores = session.user_top_scores_per_skillset("kangalioo")?;
	/// println!("kangalioo's 5th best handstream score is {:?}", top_scores.handstream[4]);
	/// ```
	pub fn user_top_scores_per_skillset(&mut self,
		username: &str,
	) -> Result<UserTopScoresPerSkillset, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}/all", username)
		))?;

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
					ssr: skillsets8_from_eo(&score_json),
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
	/// If the scorekey doesn't exist `Error::ScoreNotFound` is returned.
	/// 
	/// # Example
	/// ```
	/// let score_info = session.score_data("S65565b5bc377c6d78b60c0aecfd9e05955b4cf63")?;
	/// ```
	pub fn score_data(&mut self, scorekey: &str) -> Result<ScoreData, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/score/{}", scorekey)
		))?;

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
			ssr: skillsets8_from_eo(&json["skillsets"]),
			judgements: parse_judgements(&json["judgements"]),
			replay: parse_replay(&json["replay"])?,
			user: ScoreDataUser {
				username: json["user"]["username"].as_str().unwrap().to_owned(),
				avatar: json["user"]["avatar"].as_str().unwrap().to_owned(),
				country_code: json["user"]["countryCode"].as_str().unwrap().to_owned(),
				ssr_overall: json["user"]["Overall"].as_f64().unwrap(),
			},
		})
	}

	pub fn chart_leaderboards(&mut self, chartkey: &str) -> Result<Vec<ChartLeaderboardScore>, Error> {
		let json = self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/charts/{}/leaderboards", chartkey)
		))?;

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
				ssr: skillsets8_from_eo(&json["skillsets"]),
				judgements: parse_judgements(&json["judgements"]),
				has_replay: json["hasReplay"].as_bool().unwrap(), // API docs are wrong again
				user: ScoreDataUser {
					username: json["user"]["userName"].as_str().unwrap().to_owned(),
					avatar: json["user"]["avatar"].as_str().unwrap().to_owned(),
					country_code: json["user"]["countryCode"].as_str().unwrap().to_owned(),
					ssr_overall: json["user"]["playerRating"].as_f64().unwrap(),
				},
			});
		}

		Ok(scores)
	}

	pub fn test(&mut self) -> Result<(), Error> {
		let best_score = &self.user_top_10_scores("kangalioo")?[0];

		// println!("{:#?}", self.user_top_skillset_scores("kangalioo", Skillset7::Technical, 3)?);
		// println!("{:#?}", self.user_top_10_scores("kangalioo")?);
		// println!("{:#?}", self.user_details("kangalioo")?);
		// println!("{:#?}", self.user_latest_scores("kangalioo")?);
		// println!("{:#?}", self.user_ranks_per_skillset("kangalioo")?);
		// println!("{:#?}", self.user_top_scores_per_skillset("kangalioo")?);
		// println!("{:#?}", self.score_data(&best_score.scorekey));
		// println!("{:#?}", self.chart_leaderboards("Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2"));

		// check if wifescores are all normalized to a max of 1.0
		// println!("{} {} {} {} {} {}",
		// 	self.user_top_skillset_scores("kangalioo", Skillset7::Stream, 3)?[0].wifescore,
		// 	self.user_top_10_scores("kangalioo")?[0].wifescore,
		// 	self.user_latest_scores("kangalioo")?[0].wifescore,
		// 	self.user_top_scores_per_skillset("kangalioo")?.jackspeed[0].wifescore,
		// 	self.score_data(&best_score.scorekey)?.wifescore,
		// 	self.chart_leaderboards("Xbbff339a2c301d7bf03dc99bc1b013c3b80e75d2")?[0].wifescore,
		// );
		
		Ok(())
	}
}