mod structs;
pub use structs::*;

use etterna::*;

use crate::extension_traits::*;
use crate::Error;

/// The kind of ranges that EO can process. Ranges can never be empty! They must have one or more
/// elements
pub trait EoRange {
	#[doc(hidden)]
	/// The length must not be zero
	fn start_length(&self) -> (u32, u32);
}

impl EoRange for std::ops::Range<u32> {
	fn start_length(&self) -> (u32, u32) {
		match self.end.saturating_sub(self.start) {
			0 => panic!("Range cannot be empty or negative: {:?}", self),
			length => (self.start, length),
		}
	}
}

impl EoRange for std::ops::RangeInclusive<u32> {
	fn start_length(&self) -> (u32, u32) {
		match (self.end() + 1).saturating_sub(*self.start()) {
			0 => panic!("Range cannot be empty or negative: {:?}", self),
			length => (*self.start(), length),
		}
	}
}

impl EoRange for std::ops::RangeToInclusive<u32> {
	fn start_length(&self) -> (u32, u32) {
		(0, self.end + 1)
	}
}

impl EoRange for std::ops::RangeTo<u32> {
	fn start_length(&self) -> (u32, u32) {
		match self.end {
			0 => panic!("Range cannot be empty: {:?}", self),
			length => (0, length),
		}
	}
}

impl EoRange for std::ops::RangeFull {
	fn start_length(&self) -> (u32, u32) {
		// EO interprets a zero length as a full range
		(0, 0)
	}
}

pub struct Session {
	// Rate limiting stuff
	last_request: std::sync::Mutex<std::time::Instant>, // could replace this was smth like a AtomicInstant
	request_cooldown: std::time::Duration,

	timeout: Option<std::time::Duration>,

	http: reqwest::Client,
}

impl Session {
	pub fn new(
		request_cooldown: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Self {
		Self {
			request_cooldown,
			timeout,
			last_request: std::sync::Mutex::new(std::time::Instant::now() - request_cooldown),
			http: reqwest::Client::new(),
		}
	}

	async fn request(
		&self,
		method: reqwest::Method,
		path: &str,
		request_callback: impl Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
	) -> Result<String, Error> {
		// UNWRAP: propagate panics
		let rate_limit =
			crate::rate_limit(self.last_request.lock().unwrap(), self.request_cooldown);
		rate_limit.await;

		let mut request = self
			.http
			.request(method, &format!("https://etternaonline.com/{}", path));
		if let Some(timeout) = self.timeout {
			request = request.timeout(timeout);
		}
		request = request_callback(request);

		let response = request.send().await?.text().await?;

		if response.trim().is_empty() {
			return Err(Error::EmptyServerResponse);
		}

		Ok(response)
	}

	/// Panics if the provided range is empty or negative
	pub async fn packlist(&self, range_to_retrieve: impl EoRange) -> Result<Vec<PackEntry>, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self
			.request(reqwest::Method::POST, "pack/packlist", |r| {
				r.form(&[
					("start", &start.to_string()),
					("length", &length.to_string()),
				])
			})
			.await?;
		let json: serde_json::Value = serde_json::from_str(&json)?;

		json["data"]
			.array()?
			.iter()
			.map(|json| {
				Ok(PackEntry {
					average_msd: json["average"].attempt_get("average_msd", |j| {
						Some(j.as_str()?.extract("\" />", "</span>")?.parse().ok()?)
					})?,
					datetime: json["date"]
						.attempt_get("datetime", |j| Some(j.as_str()?.to_owned()))?,
					size: json["size"].attempt_get("size", |j| Some(j.as_str()?.parse().ok()?))?,
					name: json["packname"].attempt_get("name", |j| {
						Some(j.as_str()?.extract(">", "</a>")?.to_owned())
					})?,
					id: json["packname"].attempt_get("id", |j| {
						Some(j.as_str()?.extract("pack/", "\"")?.parse().ok()?)
					})?,
					num_votes: json["r_avg"].attempt_get("num_votes", |j| {
						Some(j.as_str()?.extract("title='", " votes")?.parse().ok()?)
					})?,
					average_vote: json["r_avg"].attempt_get("average_vote", |j| {
						Some(j.as_str()?.extract("votes'>", "</div>")?.parse().ok()?)
					})?,
					download_link: json["download"].attempt_get("download_link", |j| {
						Some(j.as_str()?.extract("href=\"", "\">")?.to_owned())
					})?,
				})
			})
			.collect()
	}

	/// Panics if the provided range is empty or negative
	pub async fn leaderboard(
		&self,
		range_to_retrieve: impl EoRange,
		sort_criterium: LeaderboardSortBy,
		sort_direction: SortDirection,
	) -> Result<Vec<LeaderboardEntry>, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self
			.request(reqwest::Method::POST, "leaderboard/leaderboard", |r| {
				r.form(&[
					("start", start.to_string().as_str()),
					("length", length.to_string().as_str()),
					(
						"order[0][dir]",
						match sort_direction {
							SortDirection::Ascending => "asc",
							SortDirection::Descending => "desc",
						},
					),
					(
						"order[0][column]",
						match sort_criterium {
							LeaderboardSortBy::Username => "1",
							LeaderboardSortBy::Rating(Skillset8::Overall) => "2",
							LeaderboardSortBy::Rating(Skillset8::Stream) => "3",
							LeaderboardSortBy::Rating(Skillset8::Jumpstream) => "4",
							LeaderboardSortBy::Rating(Skillset8::Handstream) => "5",
							LeaderboardSortBy::Rating(Skillset8::Stamina) => "6",
							LeaderboardSortBy::Rating(Skillset8::Jackspeed) => "7",
							LeaderboardSortBy::Rating(Skillset8::Chordjack) => "8",
							LeaderboardSortBy::Rating(Skillset8::Technical) => "9",
						},
					),
				])
			})
			.await?;
		let json: serde_json::Value = serde_json::from_str(&json)?;

		json["data"]
			.array()?
			.iter()
			.map(|json| {
				Ok(LeaderboardEntry {
					rank: json["rank"].attempt_get("rank int", |j| {
						Some(j.as_str()?.trim_start_matches('#').parse().ok()?)
					})?,
					username: json["username"].attempt_get("leaderboard username", |j| {
						Some(j.as_str()?.extract("/user/", "\"")?.to_owned())
					})?,
					country: (|| {
						Some(Country {
							code: json["username"]
								.as_str()?
								.extract("/img/flags/", ".svg")?
								.to_owned(),
							name: json["username"]
								.as_str()?
								.extract("title=\"", "\"")?
								.to_owned(),
						})
					})(),
					avatar: json["username"].attempt_get("leaderboard username", |j| {
						Some(j.as_str()?.extract("/avatars/", "\"")?.to_owned())
					})?,
					rating: etterna::Skillsets8 {
						overall: json["player_rating"].f32_()?,
						stamina: json["Stamina"].f32_()?,
						stream: json["Stream"].f32_()?,
						jumpstream: json["Jumpstream"].f32_()?,
						handstream: json["Handstream"].f32_()?,
						jackspeed: json["JackSpeed"].f32_()?,
						chordjack: json["Chordjack"].f32_()?,
						technical: json["Technical"].f32_()?,
					},
				})
			})
			.collect()
	}

	/// Panics if the provided range is empty or negative
	pub async fn user_scores(
		&self,
		user_id: u32,
		range_to_retrieve: impl EoRange,
		song_name_search_query: Option<&str>,
		sort_criterium: UserScoresSortBy,
		sort_direction: SortDirection,
		include_invalid: bool,
	) -> Result<UserScores, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self
			.request(
				reqwest::Method::POST,
				if include_invalid {
					"score/userScores"
				} else {
					"valid_score/userScores"
				},
				|r| {
					r.form(&[
						("start", &start.to_string() as &str),
						("length", &length.to_string()),
						("userid", &user_id.to_string()),
						(
							"order[0][dir]",
							match sort_direction {
								SortDirection::Ascending => "asc",
								SortDirection::Descending => "desc",
							},
						),
						(
							"order[0][column]",
							match sort_criterium {
								UserScoresSortBy::SongName => "0",
								UserScoresSortBy::Rate => "1",
								UserScoresSortBy::SsrOverall => "2",
								UserScoresSortBy::SsrOverallNerfed => "3",
								UserScoresSortBy::Wifescore => "4",
								UserScoresSortBy::Date => "5",
								UserScoresSortBy::Stream => "6",
								UserScoresSortBy::Jumpstream => "7",
								UserScoresSortBy::Handstream => "8",
								UserScoresSortBy::Stamina => "9",
								UserScoresSortBy::Jacks => "10",
								UserScoresSortBy::Chordjacks => "11",
								UserScoresSortBy::Technical => "12",
								UserScoresSortBy::ChordCohesion => "13",
								UserScoresSortBy::Scorekey => "",
							},
						),
						("search[value]", song_name_search_query.unwrap_or("")),
					])
				},
			)
			.await?;
		let json: serde_json::Value = serde_json::from_str(&json)?;

		let scores = json["data"]
			.array()?
			.iter()
			.map(|json| {
				Ok(UserScore {
					song_name: json["songname"].attempt_get("song name", |j| {
						Some(j.as_str()?.extract("\">", "</a>")?.to_owned())
					})?,
					song_id: json["songname"].attempt_get("song id", |j| {
						Some(j.as_str()?.extract("song/view/", "\"")?.parse().ok()?)
					})?,
					// scorekey: json["scorekey"].parse()?, // this disappeared
					rate: json["user_chart_rate_rate"].parse()?,
					wifescore: json["wifescore"].attempt_get("wifescore", |j| {
						Some(etterna::Wifescore::from_percent(
							j.as_str()?
								.extract("<span class=", "</span>")?
								.extract(">", "%")?
								.parse()
								.ok()?,
						)?)
					})?,
					judgements: json["wifescore"].attempt_get("judgements", |j| {
						let string = j.as_str()?;
						Some(etterna::TapJudgements {
							marvelouses: string.extract("Marvelous: ", "<br")?.parse().ok()?,
							perfects: string.extract("Perfect: ", "<br")?.parse().ok()?,
							greats: string.extract("Great: ", "<br")?.parse().ok()?,
							goods: string.extract("Good: ", "<br")?.parse().ok()?,
							bads: string.extract("Bad: ", "<br")?.parse().ok()?,
							misses: string.extract("Miss: ", "<br")?.parse().ok()?,
						})
					})?,
					date: json["datetime"].string()?,
					has_chord_cohesion: json["nocc"].attempt_get("'Off' or 'On'", |j| {
						match j.as_str()? {
							"On" => Some(true),
							"Off" => Some(false),
							_ => None,
						}
					})?,
					validity_dependant: if json["Overall"].str_()?.contains("Invalid Score") {
						None
					} else {
						Some(ValidUserScoreInfo {
							scorekey: json["Overall"].attempt_get("scorekey", |j| {
								Some(
									j.as_str()?.extract("score/view/", "\"")?[..41]
										.parse()
										.ok()?,
								)
							})?,
							user_id: json["Overall"].attempt_get("user id", |j| {
								Some(
									j.as_str()?.extract("score/view/", "\"")?[41..]
										.parse()
										.ok()?,
								)
							})?,
							// The following are zero if the score is invalid
							ssr: etterna::Skillsets8 {
								overall: json["Overall"].attempt_get("overall", |j| {
									Some(j.as_str()?.extract("\">", "<")?.parse().ok()?)
								})?,
								stream: json["stream"].parse()?,
								jumpstream: json["jumpstream"].parse()?,
								handstream: json["handstream"].parse()?,
								stamina: json["stamina"].parse()?,
								jackspeed: json["jackspeed"].parse()?,
								chordjack: json["chordjack"].parse()?,
								technical: json["technical"].parse()?,
							},
							ssr_overall_nerfed: json["Nerf"].f32_()?,
						})
					},
				})
			})
			.collect::<Result<Vec<UserScore>, Error>>()?;

		Ok(UserScores {
			entries_before_search_filtering: json["recordsTotal"].u32_()?,
			entries_after_search_filtering: json["recordsFiltered"].u32_()?,
			scores,
		})
	}

	pub async fn user_details(&self, username: &str) -> Result<UserDetails, Error> {
		let response = self
			.request(reqwest::Method::GET, &format!("user/{}", username), |r| r)
			.await?;

		if response.contains("Looks like the page you want, aint here.")
			|| response.contains("disallowed characters") // if username has funky chars
			|| response.contains("\"errors\":[]") // if username is empty
			|| response.is_empty()
		{
			return Err(Error::UserNotFound {
				name: Some(username.to_owned()),
			});
		}

		Ok(UserDetails {
			user_id: (|| response.as_str().extract("'userid': '", "'")?.parse().ok())()
				.ok_or_else(|| {
					Error::InvalidDataStructure("No userid found in user page".to_owned())
				})?,
			// // The following code is not yet tested
			// total_scores: (|| {
			// 	response
			// 		.as_str()
			// 		.extract("Total Scores", "</td>")?
			// 		.extract("<td>", "</td>")?
			// 		.parse()
			// 		.ok()
			// })()
			// .ok_or_else(|| {
			// 	Error::InvalidDataStructure("Couldn't find total scores in user page".to_owned())
			// }),
			// unique_songs: (|| {
			// 	response
			// 		.as_str()
			// 		.extract("Unique Songs Played", "</td>")?
			// 		.extract("<td>", "</td>")?
			// 		.parse()
			// 		.ok()
			// })()
			// .ok_or_else(|| {
			// 	Error::InvalidDataStructure("Couldn't find total scores in user page".to_owned())
			// }),
		})
	}

	/// `all_rates` - if true, show users' scores for all rates instead of just their best score
	pub async fn chart_leaderboard(
		&self,
		chartkey: impl AsRef<str>,
		range_to_retrieve: impl EoRange,
		user_name_search_query: Option<&str>,
		sort_criterium: ChartLeaderboardSortBy,
		sort_direction: SortDirection,
		all_rates: bool,
		include_invalid: bool,
	) -> Result<ChartLeaderboard, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self
			.request(
				reqwest::Method::POST,
				if include_invalid {
					"score/chartOverallScores"
				} else {
					"valid_score/chartOverallScores"
				},
				|r| {
					r.form(&[
						("start", &start.to_string() as &str),
						("length", &length.to_string()),
						("chartkey", chartkey.as_ref()),
						("top", if all_rates { "" } else { "true" }),
						(
							"order[0][dir]",
							match sort_direction {
								SortDirection::Ascending => "asc",
								SortDirection::Descending => "desc",
							},
						),
						(
							"order[0][column]",
							match sort_criterium {
								ChartLeaderboardSortBy::Username => "1",
								ChartLeaderboardSortBy::SsrOverall => "2",
								ChartLeaderboardSortBy::Rate => "4",
								ChartLeaderboardSortBy::Wife => "5",
								ChartLeaderboardSortBy::Date => "6",
								ChartLeaderboardSortBy::Marvelouses => "7",
								ChartLeaderboardSortBy::Perfects => "8",
								ChartLeaderboardSortBy::Greats => "9",
								ChartLeaderboardSortBy::Goods => "10",
								ChartLeaderboardSortBy::Bads => "11",
								ChartLeaderboardSortBy::Misses => "12",
								ChartLeaderboardSortBy::MaxCombo => "13",
								ChartLeaderboardSortBy::Scorekey => "",
							},
						),
						("search[value]", user_name_search_query.unwrap_or("")),
					])
				},
			)
			.await?;
		let json: serde_json::Value = serde_json::from_str(&json)?;

		Ok(ChartLeaderboard {
			entries_before_search_filtering: json["recordsTotal"].u32_()?,
			entries_after_search_filtering: json["recordsFiltered"].u32_()?,
			entries: json["data"]
				.array()?
				.iter()
				.map(|json| {
					Ok(ChartLeaderboardEntry {
						// turns out this is actually not a rank but just an index, i.e. if you sort by
						// date, rank #1 would be the latest score, not the best score. _That_ kind of rank
						// is pretty useless so let's not parse it to not confuse users about what this is
						// rank: json.attempt_get("rank string", |json| {
						// 	let s = json["rank"].as_str()?;
						// 	if &s[0..1] != "#" { return None; }
						// 	Some(s[1..].parse::<u32>().ok()? - 1)
						// })?,
						date: json["date"].string()?,
						judgements: TapJudgements {
							marvelouses: json["marv"].parse()?,
							perfects: json["perfect"].parse()?,
							greats: json["great"].parse()?,
							goods: json["good"].parse()?,
							bads: json["bad"].parse()?,
							misses: json["miss"].parse()?,
						},
						max_combo: json["combo"].parse()?,
						rate: json["rate"].parse()?,
						ssr_overall: json["score"].attempt_get("SSR from score html", |json| {
							Some(json.as_str()?.extract("\">", "<")?.parse().ok()?)
						})?,
						ssr_overall_nerfed: json["nerf"].f32_()?,
						scorekey: json["score"]
							.attempt_get("scorekey from score html", |json| {
								Some(json.as_str()?.extract("view/", "\"")?[..41].parse().ok()?)
							})?,
						user_id: json["score"].attempt_get("scorekey from score html", |json| {
							Some(json.as_str()?.extract("view/", "\"")?[41..].parse().ok()?)
						})?,
						username: json["username"]
							.attempt_get("username from username html", |json| {
								Some(json.as_str()?.extract("user/", "\"")?.to_owned())
							})?,
						wifescore: json["wife"].attempt_get(
							"wifescore from wife html",
							|json| {
								Some(Wifescore::from_percent(
									json.as_str()?.extract(">", "%")?.parse::<f32>().ok()?,
								)?)
							},
						)?,
					})
				})
				.collect::<Result<Vec<_>, Error>>()?,
		})
	}
}
