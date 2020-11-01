mod structs;
pub use structs::*;

use etterna::*;

use crate::Error;
use crate::extension_traits::*;

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
}

impl Session {
	pub fn new(
		request_cooldown: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Self {
		Self {
			request_cooldown, timeout,
			last_request: std::sync::Mutex::new(std::time::Instant::now() - request_cooldown),
		}
	}

	fn request(&self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response,
	) -> Result<ureq::Response, Error> {
		// UNWRAP: propagate panics
		crate::rate_limit(&mut *self.last_request.lock().unwrap(), self.request_cooldown);

		let mut request = ureq::request(method, &format!("https://etternaonline.com/{}", path));
		if let Some(timeout) = self.timeout {
			request.timeout(timeout);
		}
		let response = request_callback(request);

		Ok(response)
	}

	/// Panics if the provided range is empty or negative
	pub fn packlist(&self,
		range_to_retrieve: impl EoRange,
	) -> Result<Vec<PackEntry>, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self.request("POST", "pack/packlist", |mut r| r
			.send_form(&[
				("start", &start.to_string()),
				("length", &length.to_string()),
			])
		)?.into_json()?;

		json["data"].array()?.iter().map(|json| Ok(PackEntry {
			average_msd: json["average"].str_()?
				.extract("\" />", "</span>").json_unwrap()?
				.parse().json_unwrap()?,
			datetime: json["date"].str_()?
				.to_owned(),
			size: json["size"].str_()?
				.parse().json_unwrap()?,
			name: json["packname"].str_()?
				.extract(">", "</a>").json_unwrap()?
				.to_owned(),
			id: json["packname"].str_()?
				.extract("pack/", "\"").json_unwrap()?
				.parse().json_unwrap()?,
			num_votes: json["r_avg"].str_()?
				.extract("title='", " votes").json_unwrap()?
				.parse().json_unwrap()?,
			average_vote: json["r_avg"].str_()?
				.extract("votes'>", "</div>").json_unwrap()?
				.parse().json_unwrap()?,
			download_link: json["download"].str_()?
				.extract("href=\"", "\">").json_unwrap()?
				.to_owned(),
		})).collect()
	}

	/// Panics if the provided range is empty or negative
	pub fn leaderboard(&self,
		range_to_retrieve: impl EoRange,
	) -> Result<Vec<LeaderboardEntry>, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self.request("POST", "leaderboard/leaderboard", |mut r| r
			.send_form(&[
				("start", &start.to_string()),
				("length", &length.to_string()),
			])
		)?.into_json()?;

		json["data"].array()?.iter().map(|json| {
			let user_string = json["username"].str_()?;

			Ok(LeaderboardEntry {
				rank: json["rank"].attempt_get("rank int", |j| {
					j.as_str()?.trim_start_matches('#').parse().ok()
				})?,
				username: user_string
					.extract("/user/", "\"").json_unwrap()?
					.to_owned(),
				country: (|| Some(Country {
					code: user_string
						.extract("/img/flags/", ".svg")?
						.to_owned(),
					name: user_string
						.extract("title=\"", "\"")?
						.to_owned(),
				}))(),
				avatar: user_string
					.extract("/avatars/", "\"").json_unwrap()?
					.to_owned(),
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
		}).collect()
	}

	/// Panics if the provided range is empty or negative
	pub fn user_scores(&self,
		user_id: u32,
		range_to_retrieve: impl EoRange,
		song_name_search_query: Option<&str>,
		sort_criterium: UserScoresSortBy,
		sort_direction: SortDirection,
		include_invalid: bool,
	) -> Result<UserScores, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self.request(
			"POST",
			if include_invalid { "score/userScores" } else { "valid_score/userScores" },
			|mut r| r.send_form(&[
				("start", &start.to_string()),
				("length", &length.to_string()),
				("userid", &user_id.to_string()),
				("order[0][dir]", match sort_direction {
					SortDirection::Ascending => "asc",
					SortDirection::Descending => "desc",
				}),
				("order[0][column]", match sort_criterium {
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
				}),
				("search[value]", song_name_search_query.unwrap_or("")),
			])
		)?.into_json()?;

		let scores = json["data"].array()?.iter().map(|json| Ok(UserScore {
			song_name: json["songname"].attempt_get("song name", |j| Some(j
				.as_str()?
				.extract("\">", "</a>")?
				.to_owned()
			))?,
			song_id: json["songname"].attempt_get("song id", |j| Some(j
				.as_str()?
				.extract("song/view/", "\"")?
				.parse().ok()?
			))?,
			// scorekey: json["scorekey"].parse()?, // this disappeared
			rate: json["user_chart_rate_rate"].parse()?,
			wifescore: json["wifescore"].attempt_get("wifescore", |j| Some(
				etterna::Wifescore::from_percent(j
					.as_str()?
					.extract("<span class=", "</span>")?
					.extract(">", "%")?
					.parse().ok()?
				)?
			))?,
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
			has_chord_cohesion: json["nocc"].attempt_get("'Off' or 'On'", |j| match j.as_str()? {
				"On" => Some(true),
				"Off" => Some(false),
				_ => None,
			})?,
			validity_dependant: if json["Overall"].str_()?.contains("Invalid Score") {
				None
			} else {
				Some(ValidUserScoreInfo {
					scorekey: json["Overall"].attempt_get("scorekey", |j| Some(j
						.as_str()?
						.extract("score/view/", "\"")?
						[..41]
						.parse().ok()?
					))?,
					user_id: json["Overall"].attempt_get("user id", |j| Some(j
						.as_str()?
						.extract("score/view/", "\"")?
						[41..]
						.parse().ok()?
					))?,
					// The following are zero if the score is invalid
					ssr: etterna::Skillsets8 {
						overall: json["Overall"].attempt_get("overall", |j| Some(j
							.as_str()?
							.extract("\">", "<")?
							.parse().ok()?
						))?,
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
			}
		})).collect::<Result<Vec<UserScore>, Error>>()?;

		Ok(UserScores {
			entries_before_search_filtering: json["recordsTotal"].u32_()?,
			entries_after_search_filtering: json["recordsFiltered"].u32_()?,
			scores,
		})
	}

	pub fn user_details(&self, username: &str) -> Result<UserDetails, Error> {
		let response = self.request("GET", &format!("user/{}", username), |mut r| r.call())?;
		let response = response.into_string()?;

		if response.contains("Looks like the page you want, aint here.")
			|| response.contains("disallowed characters") // if username has funky chars
			|| response.contains("\"errors\":[]") // if username is empty
			|| response.is_empty() // or idek
		{
			return Err(Error::UserNotFound);
		}

		Ok(UserDetails {
			user_id: (|| response.as_str().extract("'userid': '", "'")?.parse().ok())()	
				.ok_or_else(|| Error::UnexpectedResponse("No userid found in user page".to_owned()))?,
		})
	}

	/// `all_rates` - if true, show users' scores for all rates instead of just their best score
	pub fn chart_leaderboard(&self,
		chartkey: impl AsRef<str>,
		range_to_retrieve: impl EoRange,
		user_name_search_query: Option<&str>,
		sort_criterium: ChartLeaderboardSortBy,
		sort_direction: SortDirection,
		all_rates: bool,
		include_invalid: bool,
	) -> Result<ChartLeaderboard, Error> {
		let (start, length) = range_to_retrieve.start_length();

		let json = self.request(
			"POST",
			if include_invalid {
				"score/chartOverallScores"
			} else {
				"valid_score/chartOverallScores"
			},
			|mut r| r.send_form(&[
				("start", &start.to_string()),
				("length", &length.to_string()),
				("chartkey", chartkey.as_ref()),
				("top", if all_rates { "" } else { "true" }),
				("order[0][dir]", match sort_direction {
					SortDirection::Ascending => "asc",
					SortDirection::Descending => "desc",
				}),
				("order[0][column]", match sort_criterium {
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
				}),
				("search[value]", user_name_search_query.unwrap_or("")),
			])
		)?.into_json()?;

		Ok(ChartLeaderboard {
			entries_before_search_filtering: json["recordsTotal"].u32_()?,
			entries_after_search_filtering: json["recordsFiltered"].u32_()?,
			entries: json["data"].array()?.iter().map(|json| Ok(ChartLeaderboardEntry {
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
				ssr_overall: json["score"].attempt_get("SSR from score html", |json| Some(
					json.as_str()?.extract("\">", "<")?.parse().ok()?
				))?,
				ssr_overall_nerfed: json["nerf"].f32_()?,
				scorekey: json["score"].attempt_get("scorekey from score html", |json| {
					Some(json.as_str()?.extract("view/", "\"")?[..41].parse().ok()?)
				})?,
				user_id: json["score"].attempt_get("scorekey from score html", |json| {
					Some(json.as_str()?.extract("view/", "\"")?[41..].parse().ok()?)
				})?,
				username: json["username"].attempt_get("username from username html", |json| Some(
					json.as_str()?.extract("user/", "\"")?.to_owned()
				))?,
				wifescore: json["wife"].attempt_get("wifescore from wife html", |json| Some(
					Wifescore::from_percent(json.as_str()?.extract(">", "%")?.parse::<f32>().ok()?)?
				))?,
			})).collect::<Result<Vec<_>, Error>>()?,
		})
	}
}