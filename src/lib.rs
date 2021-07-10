#![allow(
	clippy::len_zero,
	clippy::tabs_in_doc_comments,
	clippy::collapsible_if,
	clippy::needless_bool,
	clippy::too_many_arguments,
	clippy::needless_question_mark, // thats just what we gotta do for lack of try blocks
)]

/*!
This crate provides an ergonomic wrapper around the v1, v2 and web API of
[EtternaOnline](https://etternaonline.com), commonly abbreviated "EO" (web API is work-in-progress).

Depending on which API you choose, you might need an API token.

# Notes
Etterna terminology:
- The calculated difficulty for a chart is called MSD: Mina standardized difficulty.
- The score rating - which is variable depending on your wifescore - is called SSR:
  score-specific-rating

# Usage
For detailed usage documentation, see [`v1::Session`] and [`v2::Session`]
*/

#[cfg(feature = "serde")]
extern crate serde_ as serde;

mod extension_traits;
#[macro_use]
mod common;
pub use common::structs::*;
pub mod v1;
pub mod v2;
pub mod web;

#[doc(hidden)]
#[macro_export]
macro_rules! assert_send_future {
	($fn_result:expr) => {
		fn _assert_send_future() {
			fn a<T: Send>(_: T) {}
			#[allow(warnings, clippy::all)]
			a($fn_result);
		}
	};
}

/// Ensures that compilation fails if common functions return non-Send futures
///
/// Rust doesn't have a way to enforce async function futures implementing Send so we need this hack
fn _assert_send_future() {
	fn dummy<T>() -> T {
		panic!()
	}
	fn assert_send<T: Send>(_: T) {}

	assert_send(v1::Session::user_data(dummy(), dummy()));
	assert_send(v2::Session::user_details(dummy(), dummy()));
	assert_send(web::Session::user_details(dummy(), dummy()));
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	// Client errors
	UserNotFound { name: Option<String> },
	InvalidLogin,
	ScoreNotFound,
	SongNotFound,
	ChartNotTracked,
	ChartAlreadyFavorited,
	DatabaseError,
	GoalAlreadyExists,
	ChartAlreadyAdded,
	InvalidXml,
	NoUsersFound,

	// External errors
	Http(reqwest::Error),
	NetworkError(std::io::Error),
	InternalServerError { status_code: u16 },
	InvalidJson(serde_json::Error),
	UnknownApiError(String),
	InvalidDataStructure(String),
	EmptyServerResponse,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			// Client errors
			Self::UserNotFound { name: Some(name) } => write!(f, "User '{}' not found", name),
			Self::UserNotFound { name: None } => write!(f, "User not found"),
			Self::InvalidLogin => write!(f, "Username and password combination not found"),
			Self::ScoreNotFound => write!(f, "Score not found"),
			Self::SongNotFound => write!(f, "Song not found"),
			Self::ChartNotTracked => write!(f, "Chart not tracked"),
			Self::ChartAlreadyFavorited => write!(f, "Favorite already exists"),
			Self::DatabaseError => write!(f, "Database error"),
			Self::GoalAlreadyExists => write!(f, "Goal already exists"),
			Self::ChartAlreadyAdded => write!(f, "Chart already exists"),
			Self::InvalidXml => write!(f, "The uploaded file is not a valid XML file"),
			Self::NoUsersFound => write!(f, "No users registered"),

			// External errors
			Self::Http(e) => write!(f, "HTTP error: {}", e),
			Self::NetworkError(e) => write!(f, "General network error: {}", e),
			Self::InternalServerError { status_code } => write!(
				f,
				"Internal EtternaOnline server error (HTTP {})",
				status_code
			),
			Self::InvalidJson(e) => {
				write!(f, "Error while parsing the json sent by the server ({})", e)
			}
			Self::UnknownApiError(e) => write!(
				f,
				"Server responded to query with an unrecognized error message ({})",
				e
			),
			Self::InvalidDataStructure(e) => write!(
				f,
				"Server sent a payload that doesn't match expectations (debug: {:?})",
				e
			),
			Self::EmptyServerResponse => write!(f, "Server response was empty"),
		}
	}
}

impl From<reqwest::Error> for Error {
	fn from(mut e: reqwest::Error) -> Self {
		e.delete_url(); // let's not leak API keys
		Self::Http(e)
	}
}

macro_rules! error_from {
	($($variant:ident ( $inner:ty ) ),* $(,)?) => {
		$(
			impl From<$inner> for Error {
				fn from(e: $inner) -> Self {
					Self::$variant(e)
				}
			}
		)*
	};
}

macro_rules! error_source {
	($($variant:ident),* $(,)?) => {
		impl std::error::Error for Error {
			fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
				match self {
					$(
						Self::$variant(e) => Some(e),
					)*
					_ => None,
				}
			}
		}
	};
}

error_from! {
	NetworkError(std::io::Error),
	InvalidJson(serde_json::Error),
}
error_source! {
	Http,
	NetworkError,
	InvalidJson,
}

/// Contains context about the request which is used in error messages
#[derive(Default, Debug)]
struct RequestContext<'a> {
	user: Option<&'a str>,
	// TODO: add chartkey, scorekey, maybe country code? (if the need for better error messages arises)
}

fn rate_limit(
	mut last_request: std::sync::MutexGuard<'_, std::time::Instant>,
	request_cooldown: std::time::Duration,
) -> impl std::future::Future<Output = ()> + Send + Sync {
	let earliest_allowed_next_request = *last_request + request_cooldown;
	let wake_up_time = Ord::max(std::time::Instant::now(), earliest_allowed_next_request);

	// Assign the "last" request time before sleeping so that incoming requests while we're sleeping
	// incorporate our soon-to-be request into their rate limiting
	*last_request = wake_up_time;
	tokio::time::sleep_until(wake_up_time.into())
}

/// This only works with 4k replays at the moment! All notes beyond the first four columns are
/// discarded
///
/// If the replay doesn't have sufficient information, None is returned (see
/// [`Replay::split_into_lanes`])
///
/// Panics if the replay contains NaN
pub fn rescore<S, W>(
	replay: &Replay,
	num_hit_mines: u32,
	num_dropped_holds: u32,
	judge: &etterna::Judge,
) -> Option<etterna::Wifescore>
where
	S: etterna::ScoringSystem,
	W: etterna::Wife,
{
	let mut lanes = replay.split_into_lanes()?;

	// Yes it's correct that I'm sorting the two lists separately, and yes it's correct
	// that with that, their ordering won't be the same anymore. This is all okay, because that's
	// how the rescorers accept their data and how they work.
	for lane in lanes.iter_mut() {
		// UNWRAP: documented panic behavior
		lane.note_seconds.sort_by(|a, b| a.partial_cmp(b).unwrap());
		lane.hit_seconds.sort_by(|a, b| a.partial_cmp(b).unwrap());
	}

	Some(etterna::rescore::<S, W>(
		&lanes,
		num_hit_mines,
		num_dropped_holds,
		judge,
	))
}
