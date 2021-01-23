mod structs;
pub use structs::*;

use etterna::*;

use crate::extension_traits::*;
use crate::Error;

fn difficulty_from_eo(string: &str) -> Result<etterna::Difficulty, Error> {
	Ok(match string {
		"Beginner" => Difficulty::Beginner,
		"Easy" => Difficulty::Easy,
		"Medium" => Difficulty::Medium,
		"Hard" => Difficulty::Hard,
		"Challenge" => Difficulty::Challenge,
		"Edit" => Difficulty::Edit,
		other => {
			return Err(Error::InvalidDataStructure(format!(
				"Unexpected difficulty name '{}'",
				other
			)))
		}
	})
}

fn parse_judgements(json: &serde_json::Value) -> Result<etterna::FullJudgements, Error> {
	Ok(etterna::FullJudgements {
		marvelouses: json["marvelous"].u32_()?,
		perfects: json["perfect"].u32_()?,
		greats: json["great"].u32_()?,
		goods: json["good"].u32_()?,
		bads: json["bad"].u32_()?,
		misses: json["miss"].u32_()?,
		hit_mines: json["hitMines"].u32_()?,
		held_holds: json["heldHold"].u32_()?,
		let_go_holds: json["letGoHold"].u32_()?,
		missed_holds: json["missedHold"].u32_()?,
	})
}

/// EtternaOnline API session client, handles all requests to and from EtternaOnline.
///
/// This wrapper keeps care of expiring tokens by automatically logging back in when the login
/// token expires.
///
/// This session has rate-limiting built-in. Please do make use of it - the EO server is brittle and
/// funded entirely by donations.
///
/// Initialize a session using [`Session::new_from_login`]
///
/// # Example
/// ```rust,no_run
/// # fn main() -> Result<(), etternaonline_api::Error> {
/// # use etternaonline_api::v2::*;
/// let mut session = Session::new_from_login(
/// 	"<USERNAME>".into(),
/// 	"<PASSWORD>".into(),
/// 	"<CLIENT_DATA>".into(),
/// 	std::time::Duration::from_millis(2000), // Wait 2s inbetween requests
/// 	None, // No request timeout
/// )?;
///
/// println!("Details about kangalioo: {:?}", session.user_details("kangalioo")?);
///
/// let best_score = &session.user_top_10_scores("kangalioo")?[0];
/// println!(
/// 	"kangalioo's best score has {} misses",
/// 	session.score_data(&best_score.scorekey)?.judgements.misses
/// );
/// # Ok(()) }
/// ```
pub struct Session {
	// This stuff is needed for re-login
	username: String,
	password: String,
	client_data: String,

	// The auth key that we get from the server on login
	authorization: crate::common::AuthorizationManager<Option<String>>,

	// Rate limiting stuff
	last_request: std::sync::Mutex<std::time::Instant>,
	cooldown: std::time::Duration,

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
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// let mut session = Session::new_from_login(
	/// 	"kangalioo".into(),
	/// 	"<PASSWORD>".into(),
	/// 	"<CLIENT_DATA>".into(),
	/// 	std::time::Duration::from_millis(2000), // wait 2s inbetween requests
	/// 	None, // no timeout
	/// )?;
	///
	/// println!("Details about kangalioo: {:?}", session.user_details("kangalioo"));
	/// # Ok(()) }
	/// ```
	pub fn new_from_login(
		username: String,
		password: String,
		client_data: String,
		cooldown: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Result<Self, Error> {
		let session = Self {
			username,
			password,
			client_data,
			cooldown,
			timeout,
			authorization: crate::common::AuthorizationManager::new(None),
			last_request: std::sync::Mutex::new(std::time::Instant::now() - cooldown),
		};
		session.login()?;

		Ok(session)
	}

	// login again to generate a new session token
	// hmmm I wonder if there's a risk that the server won't properly generate a session token,
	// return Unauthorized, and then my client will try to login to get a fresh token, and the
	// process repeats indefinitely...? I just hope that the EO server never throws an Unauthorized
	// on login
	fn login(&self) -> Result<(), Error> {
		self.authorization.refresh(|| {
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
			)?;

			Ok(Some(format!(
				"Bearer {}",
				json["attributes"]["accessToken"].str_()?,
			)))
		})
	}

	// If `do_authorization` is set, the authorization field will be locked immutably! So if the
	// caller has a mutable lock active when calling generic_request, DONT PASS true FOR
	// do_authorization, or we'll deadlock!
	fn generic_request(
		&self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response,
		do_authorization: bool,
	) -> Result<serde_json::Value, Error> {
		// UNWRAP: propagate panics
		crate::rate_limit(&mut *self.last_request.lock().unwrap(), self.cooldown);

		let mut request = ureq::request(
			method,
			&format!("https://api.etternaonline.com/v2/{}", path),
		);
		if let Some(timeout) = self.timeout {
			request.timeout(timeout);
		}
		if do_authorization {
			let auth = self
				.authorization
				.get_authorization()
				.as_ref()
				.expect("No authorization set even though it was requested??")
				.clone();
			request.set("Authorization", &auth);
		}

		let response = request_callback(request);

		if let Some(ureq::Error::Io(io_err)) = response.synthetic_error() {
			if io_err.kind() == std::io::ErrorKind::TimedOut {
				return Err(Error::Timeout);
			}
		}

		let status = response.status();
		let response = match response.into_string() {
			Ok(response) => response,
			Err(e) => {
				return if e.to_string().contains("timed out reading response") {
					// yes, there are two places where timeouts can happen :p
					// see https://github.com/algesten/ureq/issues/119
					Err(Error::Timeout)
				} else {
					Err(e.into())
				};
			}
		};

		if status >= 500 {
			return Err(Error::ServerIsDown {
				status_code: status,
			});
		}

		if response.is_empty() {
			return Err(Error::EmptyServerResponse);
		}

		// only parse json if the response code is not 5xx because on 5xx response codes, the server
		// sometimes sends empty responses
		let mut json: serde_json::Value = serde_json::from_str(&response)?;

		// Error handling
		if status >= 400 {
			return match json["errors"][0]["title"].str_()? {
				"Unauthorized" => {
					// Token expired, let's login again and retry
					self.login()?;
					return self.generic_request(method, path, request_callback, do_authorization);
				}
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

	fn request(
		&self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response,
	) -> Result<serde_json::Value, Error> {
		self.generic_request(method, path, request_callback, true)
	}

	fn get(&self, path: &str) -> Result<serde_json::Value, Error> {
		self.request("GET", path, |mut request| request.call())
	}

	/// Retrieves details about the profile of the specified user.
	///
	/// Note: the aboutMe field may be an empty string
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Retrieve details about user "kangalioo"
	/// let details = session.user_details("kangalioo")?;
	/// # Ok(()) }
	/// ```
	pub fn user_details(&self, username: &str) -> Result<UserDetails, Error> {
		let json = self.get(&format!("user/{}", username))?;
		let json = &json["attributes"];

		Ok(UserDetails {
			username: json["userName"].string()?,
			about_me: json["aboutMe"].string()?,
			is_moderator: json["moderator"].bool_()?,
			is_patreon: json["patreon"].bool_()?,
			avatar_url: json["avatar"].string()?,
			country_code: json["countryCode"].string()?,
			player_rating: json["playerRating"].f32_()?,
			default_modifiers: match json["defaultModifiers"].str_()? {
				"" => None,
				modifiers => Some(modifiers.to_owned()),
			},
			rating: etterna::Skillsets8 {
				overall: json["playerRating"].f32_()?,
				stream: json["skillsets"]["Stream"].f32_()?,
				jumpstream: json["skillsets"]["Jumpstream"].f32_()?,
				handstream: json["skillsets"]["Handstream"].f32_()?,
				stamina: json["skillsets"]["Stamina"].f32_()?,
				jackspeed: json["skillsets"]["JackSpeed"].f32_()?,
				chordjack: json["skillsets"]["Chordjack"].f32_()?,
				technical: json["skillsets"]["Technical"].f32_()?,
			},
		})
	}

	fn parse_top_scores(&self, url: &str) -> Result<Vec<TopScore>, Error> {
		let json = self.get(url)?;

		json.array()?
			.iter()
			.map(|json| {
				Ok(TopScore {
					scorekey: json["id"].parse()?,
					song_name: json["attributes"]["songName"].string()?,
					ssr_overall: json["attributes"]["Overall"].f32_()?,
					wifescore: json["attributes"]["wife"].wifescore_percent_float()?,
					rate: json["attributes"]["rate"].rate_float()?,
					difficulty: json["attributes"]["difficulty"].parse()?,
					chartkey: json["attributes"]["chartKey"].parse()?,
					base_msd: etterna::Skillsets8 {
						overall: json["attributes"]["Overall"].f32_()?,
						stream: json["attributes"]["skillsets"]["Stream"].f32_()?,
						jumpstream: json["attributes"]["skillsets"]["Jumpstream"].f32_()?,
						handstream: json["attributes"]["skillsets"]["Handstream"].f32_()?,
						stamina: json["attributes"]["skillsets"]["Stamina"].f32_()?,
						jackspeed: json["attributes"]["skillsets"]["JackSpeed"].f32_()?,
						chordjack: json["attributes"]["skillsets"]["Chordjack"].f32_()?,
						technical: json["attributes"]["skillsets"]["Technical"].f32_()?,
					},
				})
			})
			.collect()
	}

	/// Retrieve the user's top scores by the given skillset. The number of scores returned is equal
	/// to `limit`
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # use etterna::*;
	/// # let mut session: Session = unimplemented!();
	/// // Retrieve the top 10 chordjack scores of user "kangalioo"
	/// let scores = session.user_top_skillset_scores("kangalioo", Skillset7::Chordjack, 10)?;
	/// # Ok(()) }
	/// ```
	pub fn user_top_skillset_scores(
		&self,
		username: &str,
		skillset: etterna::Skillset7,
		limit: u32,
	) -> Result<Vec<TopScore>, Error> {
		self.parse_top_scores(&format!(
			"user/{}/top/{}/{}",
			username,
			crate::common::skillset_to_eo(skillset),
			limit
		))
	}

	/// Retrieve the user's top 10 scores, sorted by the overall SSR. Due to a bug in the EO v2 API,
	/// it's unfortunately not possible to control the number of scores returned.
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Retrieve the top 10 scores of user "kangalioo"
	/// let scores = session.user_top_10_scores("kangalioo")?;
	/// # Ok(()) }
	/// ```
	pub fn user_top_10_scores(&self, username: &str) -> Result<Vec<TopScore>, Error> {
		self.parse_top_scores(&format!("user/{}/top//", username))
	}

	/// Retrieve the user's latest 10 scores.
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Retrieve the latest 10 scores of user "kangalioo"
	/// let scores = session.user_latest_scores("kangalioo")?;
	/// # Ok(()) }
	/// ```
	pub fn user_latest_scores(&self, username: &str) -> Result<Vec<LatestScore>, Error> {
		let json = self.get(&format!("user/{}/latest", username))?;

		json.array()?
			.iter()
			.map(|json| {
				Ok(LatestScore {
					scorekey: json["id"].parse()?,
					song_name: json["attributes"]["songName"].string()?,
					ssr_overall: json["attributes"]["Overall"].f32_()?,
					wifescore: json["attributes"]["wife"].wifescore_percent_float()?,
					rate: json["attributes"]["rate"].rate_float()?,
					difficulty: difficulty_from_eo(json["attributes"]["difficulty"].str_()?)?,
				})
			})
			.collect()
	}

	/// Retrieve the user's rank for each skillset.
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Retrieve "kangalioo"'s rank for each skillset
	/// let scores = session.user_ranks_per_skillset("kangalioo")?;
	/// # Ok(()) }
	/// ```
	pub fn user_ranks_per_skillset(&self, username: &str) -> Result<etterna::UserRank, Error> {
		let json = self.get(&format!("user/{}/ranks", username))?;
		let json = &json["attributes"];

		Ok(etterna::UserRank {
			overall: json["Overall"].u32_()?,
			stream: json["Stream"].u32_()?,
			jumpstream: json["Jumpstream"].u32_()?,
			handstream: json["Handstream"].u32_()?,
			stamina: json["Stamina"].u32_()?,
			jackspeed: json["JackSpeed"].u32_()?,
			chordjack: json["Chordjack"].u32_()?,
			technical: json["Technical"].u32_()?,
		})
	}

	/// Retrieve the user's best scores for each skillset. The number of scores yielded is not
	/// documented in the EO API, but according to my experiments it's 25.
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let top_scores = session.user_top_scores_per_skillset("kangalioo")?;
	/// println!("kangalioo's 5th best handstream score is {:?}", top_scores.handstream[4]);
	/// # Ok(()) }
	/// ```
	pub fn user_top_scores_per_skillset(
		&self,
		username: &str,
	) -> Result<UserTopScoresPerSkillset, Error> {
		let json = self.get(&format!("user/{}/all", username))?;

		let parse_skillset_top_scores = |array: &serde_json::Value| -> Result<Vec<_>, Error> {
			array
				.array()?
				.iter()
				.map(|json| {
					Ok(TopScorePerSkillset {
						song_name: json["songname"].string()?,
						rate: json["user_chart_rate_rate"].rate_float()?,
						wifescore: json["wifescore"].wifescore_proportion_float()?,
						chartkey: json["chartkey"].parse()?,
						scorekey: json["scorekey"].parse()?,
						difficulty: difficulty_from_eo(json["difficulty"].str_()?)?,
						ssr: etterna::Skillsets8 {
							overall: json["Overall"].f32_()?,
							stream: json["Stream"].f32_()?,
							jumpstream: json["Jumpstream"].f32_()?,
							handstream: json["Handstream"].f32_()?,
							stamina: json["Stamina"].f32_()?,
							jackspeed: json["JackSpeed"].f32_()?,
							chordjack: json["Chordjack"].f32_()?,
							technical: json["Technical"].f32_()?,
						},
					})
				})
				.collect()
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
	/// - panics if the passed in scorekey is in an invalid format (only applies if passed in as a
	///   `&str`, since `&Scorekey` is guaranteed to be valid)
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let score_info = session.score_data("S65565b5bc377c6d78b60c0aecfd9e05955b4cf63")?;
	/// # Ok(()) }
	/// ```
	pub fn score_data(&self, scorekey: impl AsRef<str>) -> Result<ScoreData, Error> {
		let json = self.get(&format!("score/{}", scorekey.as_ref()))?;

		let scorekey = json["id"].parse()?;
		let json = &json["attributes"];

		Ok(ScoreData {
			scorekey,
			modifiers: json["modifiers"].string()?,
			wifescore: json["wife"].wifescore_proportion_float()?,
			rate: json["rate"].rate_float()?,
			max_combo: json["maxCombo"].u32_()?,
			is_valid: json["valid"].bool_()?,
			has_chord_cohesion: !json["nocc"].bool_()?,
			song_name: json["song"]["songName"].string()?,
			artist: json["song"]["artist"].string()?,
			song_id: json["song"]["id"].u32_()?,
			ssr: etterna::Skillsets8 {
				overall: json["skillsets"]["Overall"].f32_()?,
				stream: json["skillsets"]["Stream"].f32_()?,
				jumpstream: json["skillsets"]["Jumpstream"].f32_()?,
				handstream: json["skillsets"]["Handstream"].f32_()?,
				stamina: json["skillsets"]["Stamina"].f32_()?,
				jackspeed: json["skillsets"]["JackSpeed"].f32_()?,
				chordjack: json["skillsets"]["Chordjack"].f32_()?,
				technical: json["skillsets"]["Technical"].f32_()?,
			},
			judgements: parse_judgements(&json["judgements"])?,
			replay: crate::common::parse_replay(&json["replay"])?,
			user: ScoreUser {
				username: json["user"]["username"].string()?,
				avatar: json["user"]["avatar"].string()?,
				country_code: json["user"]["countryCode"].string()?,
				overall_rating: json["user"]["Overall"].f32_()?,
			},
		})
	}

	/// Retrieves the leaderboard for the specified chart. The return type is a vector of
	/// leaderboard entries.
	///
	/// # Errors
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let leaderboard = session.chart_leaderboard("X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	///
	/// println!("The best Game Time score is being held by {}", leaderboard[0].user.username);
	/// # Ok(()) }
	/// ```
	pub fn chart_leaderboard(
		&self,
		chartkey: impl AsRef<str>,
	) -> Result<Vec<ChartLeaderboardScore>, Error> {
		let json = self.get(&format!("charts/{}/leaderboards", chartkey.as_ref()))?;

		json.array()?
			.iter()
			.map(|json| {
				Ok(ChartLeaderboardScore {
					scorekey: json["id"].parse()?,
					wifescore: json["attributes"]["wife"].wifescore_percent_float()?,
					max_combo: json["attributes"]["maxCombo"].u32_()?,
					is_valid: json["attributes"]["valid"].bool_()?,
					modifiers: json["attributes"]["modifiers"].string()?,
					has_chord_cohesion: !json["attributes"]["noCC"].bool_()?,
					rate: json["attributes"]["rate"].rate_float()?,
					datetime: json["attributes"]["datetime"].string()?,
					ssr: etterna::Skillsets8 {
						overall: json["attributes"]["skillsets"]["Overall"].f32_()?,
						stream: json["attributes"]["skillsets"]["Stream"].f32_()?,
						jumpstream: json["attributes"]["skillsets"]["Jumpstream"].f32_()?,
						handstream: json["attributes"]["skillsets"]["Handstream"].f32_()?,
						stamina: json["attributes"]["skillsets"]["Stamina"].f32_()?,
						jackspeed: json["attributes"]["skillsets"]["JackSpeed"].f32_()?,
						chordjack: json["attributes"]["skillsets"]["Chordjack"].f32_()?,
						technical: json["attributes"]["skillsets"]["Technical"].f32_()?,
					},
					judgements: parse_judgements(&json["attributes"]["judgements"])?,
					has_replay: json["attributes"]["hasReplay"].bool_()?, // API docs are wrong again
					user: ScoreUser {
						username: json["attributes"]["user"]["userName"].string()?,
						avatar: json["attributes"]["user"]["avatar"].string()?,
						country_code: json["attributes"]["user"]["countryCode"].string()?,
						overall_rating: json["attributes"]["user"]["playerRating"].f32_()?,
					},
				})
			})
			.collect()
	}

	/// Retrieves the player leaderboard for the given country.
	///
	/// # Errors
	/// - [`Error::NoUsersFound`] if there are no users registered in this country
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let leaderboard = session.country_leaderboard("DE")?;
	///
	/// println!(
	/// 	"The best German Etterna player is {} with a rating of {}",
	/// 	leaderboard[0].user.username,
	/// 	leaderboard[0].rating.overall(),
	/// );
	/// # Ok(()) }
	/// ```
	pub fn country_leaderboard(&self, country_code: &str) -> Result<Vec<LeaderboardEntry>, Error> {
		let json = self.get(&format!("leaderboard/{}", country_code))?;

		json.array()?
			.iter()
			.map(|json| {
				Ok(LeaderboardEntry {
					user: ScoreUser {
						username: json["attributes"]["user"]["username"].string()?,
						avatar: json["attributes"]["user"]["avatar"].string()?,
						country_code: json["attributes"]["user"]["countryCode"].string()?,
						overall_rating: json["attributes"]["user"]["Overall"].f32_()?,
					},
					rating: etterna::Skillsets8 {
						overall: json["attributes"]["user"]["Overall"].f32_()?,
						stream: json["attributes"]["skillsets"]["Stream"].f32_()?,
						jumpstream: json["attributes"]["skillsets"]["Jumpstream"].f32_()?,
						handstream: json["attributes"]["skillsets"]["Handstream"].f32_()?,
						stamina: json["attributes"]["skillsets"]["Stamina"].f32_()?,
						jackspeed: json["attributes"]["skillsets"]["JackSpeed"].f32_()?,
						chordjack: json["attributes"]["skillsets"]["Chordjack"].f32_()?,
						technical: json["attributes"]["skillsets"]["Technical"].f32_()?,
					},
				})
			})
			.collect()
	}

	/// Retrieves the worldwide leaderboard of players.
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let leaderboard = session.world_leaderboard()?;
	///
	/// println!(
	/// 	"The world's best Etterna player is {} with a rating of {}",
	/// 	leaderboard[0].user.username,
	/// 	leaderboard[0].rating.overall(),
	/// );
	/// # Ok(()) }
	/// ```
	pub fn world_leaderboard(&self) -> Result<Vec<LeaderboardEntry>, Error> {
		self.country_leaderboard("")
	}

	/// Retrieves the user's favorites. Returns a vector of chartkeys.
	///
	/// # Errors
	/// - [`Error::UserNotFound`] if the supplied username was not found
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let favorites = session.user_favorites("kangalioo")?;
	/// println!("kangalioo has {} favorites", favorites.len());
	/// # Ok(()) }
	/// ```
	pub fn user_favorites(&self, username: &str) -> Result<Vec<String>, Error> {
		let json = self.get(&format!("user/{}/favorites", username))?;

		json.array()?
			.iter()
			.map(|obj| Ok(obj["attributes"]["chartkey"].string()?))
			.collect()
	}

	/// Add a chart to the user's favorites.
	///
	/// # Errors
	/// - [`Error::ChartAlreadyFavorited`] if the chart is already in the user's favorites
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Favorite Game Time
	/// session.add_user_favorite("kangalioo", "X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	/// # Ok(()) }
	/// ```
	pub fn add_user_favorite(
		&self,
		username: &str,
		chartkey: impl AsRef<str>,
	) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/favorites", username),
			|mut req| req.send_form(&[("chartkey", chartkey.as_ref())]),
		)?;

		Ok(())
	}

	/// Remove a chart from the user's favorites.
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Unfavorite Game Time
	/// session.remove_user_favorite("kangalioo", "X4a15f62b66a80b62ec64521704f98c6c03d98e03")?;
	/// # Ok(()) }
	/// ```
	pub fn remove_user_favorite(
		&self,
		username: &str,
		chartkey: impl AsRef<str>,
	) -> Result<(), Error> {
		self.request(
			"DELETE",
			&format!("user/{}/favorites/{}", username, chartkey.as_ref()),
			|mut request| request.call(),
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
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// let score_goals = session.user_goals("theropfather")?;
	///
	/// println!("theropfather has {} goals", score_goals.len());
	/// # Ok(()) }
	/// ```
	pub fn user_goals(&self, username: &str) -> Result<Vec<ScoreGoal>, Error> {
		let json = self.get(&format!("user/{}/goals", username))?;

		json.array()?
			.iter()
			.map(|json| {
				Ok(ScoreGoal {
					chartkey: json["attributes"]["chartkey"].parse()?,
					rate: json["attributes"]["rate"].rate_float()?,
					wifescore: json["attributes"]["wife"].wifescore_proportion_float()?,
					time_assigned: json["attributes"]["timeAssigned"].string()?,
					time_achieved: if json["attributes"]["achieved"].bool_int()? {
						Some(json["attributes"]["timeAchieved"].string()?)
					} else {
						None
					},
				})
			})
			.collect()
	}

	/// Add a new score goal.
	///
	/// # Errors
	/// - [`Error::GoalAlreadyExists`] when the goal already exists in the database
	/// - [`Error::ChartNotTracked`] if the chartkey provided is not tracked by EO
	/// - [`Error::DatabaseError`] if there was a problem with the database
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
	/// // Add a Game Time 1.0x AA score goal
	/// session.add_user_goal(
	/// 	"kangalioo",
	/// 	"X4a15f62b66a80b62ec64521704f98c6c03d98e03",
	/// 	1.0,
	/// 	0.93,
	/// 	"2020-07-13 22:48:26",
	/// )?;
	/// # Ok(()) }
	/// ```
	// TODO: somehow enforce that `time_assigned` is valid ISO 8601
	pub fn add_user_goal(
		&self,
		username: &str,
		chartkey: impl AsRef<str>,
		rate: f64,
		wifescore: f64,
		time_assigned: &str,
	) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/goals", username),
			|mut request| {
				request.send_form(&[
					("chartkey", chartkey.as_ref()),
					("rate", &format!("{}", rate)),
					("wife", &format!("{}", wifescore)),
					("timeAssigned", time_assigned),
				])
			},
		)?;

		Ok(())
	}

	/// Remove the user goal with the specified chartkey, rate and wifescore.
	///
	/// Note: this API call doesn't seem to do anything
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # let mut session: Session = unimplemented!();
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
	/// # Ok(()) }
	/// ```
	pub fn remove_user_goal(
		&self,
		username: &str,
		chartkey: impl AsRef<str>,
		rate: Rate,
		wifescore: Wifescore,
	) -> Result<(), Error> {
		self.request(
			"DELETE",
			&format!(
				"user/{}/goals/{}/{}/{}",
				username,
				chartkey.as_ref(),
				wifescore.as_proportion(),
				rate.as_f32()
			),
			|mut request| request.call(),
		)?;

		Ok(())
	}

	/// Update a score goal by replacing all its attributes with the given ones.
	///
	/// # Example
	/// ```rust,no_run
	/// # fn main() -> Result<(), etternaonline_api::Error> {
	/// # use etternaonline_api::v2::*;
	/// # use etterna::*;
	/// # let mut session: Session = unimplemented!();
	/// // Let's up kangalioo's first score goal's rate by 0.05
	///
	/// let mut score_goal = &mut session.user_goals("kangalioo")?[0];
	///
	/// // Add 0.05 to the rate
	/// score_goal.rate += Rate::from(0.05);
	///
	/// session.update_user_goal("kangalioo", score_goal)?;
	/// # Ok(()) }
	/// ```
	pub fn update_user_goal(&self, username: &str, goal: &ScoreGoal) -> Result<(), Error> {
		self.request(
			"POST",
			&format!("user/{}/goals/update", username),
			|mut request| {
				request.send_form(&[
					("chartkey", goal.chartkey.as_ref()),
					("timeAssigned", &goal.time_assigned),
					(
						"achieved",
						if goal.time_achieved.is_some() {
							"1"
						} else {
							"0"
						},
					),
					("rate", &format!("{}", goal.rate)),
					("wife", &format!("{}", goal.wifescore)),
					(
						"timeAchieved",
						goal.time_achieved
							.as_deref()
							.unwrap_or("0000-00-00 00:00:00"),
					),
				])
			},
		)?;

		Ok(())
	}

	// Let's find out how this works and properly implement it, when I finally find out how to login
	// into the fucking v2 API again >:(
	// pub fn pack_list(&self) -> Result<(), Error> {
	// 	let json = self.request("GET", "packs", |mut r| r.call())?;

	// 	println!("{:#?}", json);

	// 	Ok(())
	// }

	// pub fn test(&self) -> Result<(), Error> {
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
