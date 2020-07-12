#![allow(clippy::tabs_in_doc_comments)]

/*!
This crate provides an ergonomic wrapper around the v2 API of
[EtternaOnline](https://etternaonline.com) (commonly abbreviated "EO"). The EO API requires a valid
username and password combination to expose its functions. You will also need an API token called
"client data".
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
}

impl Error {
	fn unexpected_response_code(code: u16) -> Self {
		Self::UnexpectedResponse(format!("Unexpected response code {}", code))
	}
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

/// This is an EtternaOnline API session. It automatically keeps cares of expiring tokens by
/// automatically logging back in when the login token expires.
/// 
/// This session has rate-limiting built-in. _Please_ pass a sensible value in the realm of around a
/// section and don't circumvent this by passing an empty duration - the EO server is brittle and
/// funded entirely by donations.
/// 
/// Initialize a session using [`Session::new_from_login`]
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
	) -> Result<(u16, serde_json::Value), Error> {

		// Do tha rate-limiting
		let time_since_last_request = std::time::Instant::now().duration_since(self.last_request);
		if time_since_last_request < self.rate_limit {
			std::thread::sleep(self.rate_limit - time_since_last_request);
		}
		self.last_request = std::time::Instant::now();

		let response = builder()
			.set("Authorization", &self.authorization)
			.call();
		
		if response.status() == 401 {
			// Token expired, let's login again and retry
			self.login()?;
			self.request(builder)
		} else {
			let status = response.status();
			let json = response.into_json()
				.map_err(|e| Error::InvalidJson(format!("{}", e)))?;
			Ok((status, json))
		}
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
		let json = match self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}", username)
		))? {
			(200, json) => json,
			(404, _) => return Err(Error::UserNotFound),
			(code, _) => return Err(Error::unexpected_response_code(code)),
		};

		let json = &json["data"]["attributes"];

		Ok(UserDetails {
			username: json["userName"].as_str().unwrap().to_owned(),
			about_me: json["aboutMe"].as_str().unwrap().to_owned(),
			is_moderator: json["moderator"].as_bool().unwrap(),
			is_patreon: json["patreon"].as_bool().unwrap(),
			avatar_url: json["avatar"].as_str().unwrap().to_owned(),
			country_code: json["countryCode"].as_str().unwrap().to_owned(),
			player_rating: json["playerRating"].as_f64().unwrap(),
			default_modifiers: json["defaultModifiers"].as_str().unwrap().to_owned(),
			skillsets: Skillsets8 {
				overall: json["skillsets"]["Overall"].as_f64().unwrap(),
				stream: json["skillsets"]["Stream"].as_f64().unwrap(),
				jumpstream: json["skillsets"]["Jumpstream"].as_f64().unwrap(),
				handstream: json["skillsets"]["Handstream"].as_f64().unwrap(),
				stamina: json["skillsets"]["Stamina"].as_f64().unwrap(),
				jackspeed: json["skillsets"]["JackSpeed"].as_f64().unwrap(),
				chordjack: json["skillsets"]["Chordjack"].as_f64().unwrap(),
				technical: json["skillsets"]["Technical"].as_f64().unwrap(),
			}
		})
	}
	
	/// Retrieve the user's top scores by the given skillset. The number of scores returned is equal
	/// to `limit`
	/// 
	/// If there is no user with the specified username, `Error::UserNotFound` is returned.
	/// 
	/// # Example
	/// ```
	/// // Retrieve the top 10 chordjack scores of user "kangalioo"
	/// let scores = session.user_top_scores("kangalioo", Skillset7::Chordjack, 10)?;
	/// ```
	pub fn user_top_scores(&mut self,
		username: &str,
		// skillset: impl Into<Option<Skillset7>>, // TODO: add this back in when the EO bug has been fixed
		skillset: Skillset7,
		limit: u32,
	) -> Result<Vec<TopScore>, Error> {
		// let skillset = skillset.into().map(skillset7_to_eo).unwrap_or("");
		let skillset = skillset7_to_eo(skillset);

		let url = &format!("https://api.etternaonline.com/v2/user/{}/top/{}/{}",
			username, skillset, limit
		);
		println!("Url: {:?}", url);
		let json = match self.request(|| ureq::get(url))? {
			(200, json) => json,
			(400, _) => return Err(Error::UserNotFound),
			(404, _) => return Err(Error::UserNotFound),
			(code, _) => return Err(Error::unexpected_response_code(code)),
		};

		let mut scores = Vec::new();
		for score_json in json["data"].as_array().unwrap() {
			let difficulty = difficulty_from_eo(score_json["attributes"]["difficulty"].as_str().unwrap())?;

			let base_skillsets = Skillsets7 {
				// yes, the api really doesn't return Overall
				stream: score_json["attributes"]["skillsets"]["Stream"].as_f64().unwrap(),
				jumpstream: score_json["attributes"]["skillsets"]["Jumpstream"].as_f64().unwrap(),
				handstream: score_json["attributes"]["skillsets"]["Handstream"].as_f64().unwrap(),
				stamina: score_json["attributes"]["skillsets"]["Stamina"].as_f64().unwrap(),
				jackspeed: score_json["attributes"]["skillsets"]["JackSpeed"].as_f64().unwrap(),
				chordjack: score_json["attributes"]["skillsets"]["Chordjack"].as_f64().unwrap(),
				technical: score_json["attributes"]["skillsets"]["Technical"].as_f64().unwrap(),
			};

			scores.push(TopScore {
				scorekey: score_json["id"].as_str().unwrap().to_owned(),
				song_name: score_json["attributes"]["songName"].as_str().unwrap().to_owned(),
				ssr_overall: score_json["attributes"]["Overall"].as_f64().unwrap(),
				wifescore: score_json["attributes"]["wife"].as_f64().unwrap(),
				rate: score_json["attributes"]["rate"].as_f64().unwrap(),
				difficulty,
				chartkey: score_json["attributes"]["chartKey"].as_str().unwrap().to_owned(),
				base_skillsets,
			});
		}

		Ok(scores)
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
		let json = match self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}/latest", username)
		))? {
			(200, json) => json,
			(404, _) => return Err(Error::UserNotFound),
			(code, _) => return Err(Error::unexpected_response_code(code)),
		};

		let mut scores = Vec::new();
		for score_json in json["data"].as_array().unwrap() {
			scores.push(LatestScore {
				scorekey: score_json["id"].as_str().unwrap().to_owned(),
				song_name: score_json["attributes"]["songName"].as_str().unwrap().to_owned(),
				ssr_overall: score_json["attributes"]["Overall"].as_f64().unwrap(),
				wifescore: score_json["attributes"]["wife"].as_f64().unwrap(),
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
		let json = match self.request(|| ureq::get(
			&format!("https://api.etternaonline.com/v2/user/{}/ranks", username)
		))? {
			(200, json) => json,
			(404, _) => return Err(Error::UserNotFound),
			(code, _) => return Err(Error::unexpected_response_code(code)),
		};

		let ranks_json = &json["data"]["attributes"];
		Ok(UserRanksPerSkillset {
			overall: ranks_json["Overall"].as_i64().unwrap() as u32,
			stream: ranks_json["Stream"].as_i64().unwrap() as u32,
			jumpstream: ranks_json["Jumpstream"].as_i64().unwrap() as u32,
			handstream: ranks_json["Handstream"].as_i64().unwrap() as u32,
			stamina: ranks_json["Stamina"].as_i64().unwrap() as u32,
			jackspeed: ranks_json["JackSpeed"].as_i64().unwrap() as u32,
			chordjack: ranks_json["Chordjack"].as_i64().unwrap() as u32,
			technical: ranks_json["Technical"].as_i64().unwrap() as u32,
		})
	}

	// pub fn user_top_scores_per_skillset(&mut self,
	// 	username: &str,
	// ) -> Result<(UserTopScoresPerSkillset, UserRanksPerSkillset), Error> {
	// 	todo!(); // TODO: Waiting for information from rop
	// }

	// pub fn test(&mut self) -> Result<(), Error> {
	// 	println!("{:#?} {:#?}",
	// 		// self.user_top_scores("kangalioo", Skillset7::Technical, 5)?,
	// 		// self.user_top_scores("kangalioo", Skillset7::Jumpstream, 5)?,
	// 		// self.user_latest_scores("kangalioo"),
	// 		// self.user_latest_scores("theropfather"),
	// 		// self.user_ranks("kangalioo"),
	// 		// self.user_ranks("theropfather"),
	// 	);
		
	// 	Ok(())
	// }
}