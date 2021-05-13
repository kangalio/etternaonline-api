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

thiserror_lite::err_enum! {
	#[derive(Debug)]
	#[non_exhaustive]
	pub enum Error {
		// Client errors
		#[error("User not found")]
		UserNotFound,
		#[error("Username and password combination not found")]
		InvalidLogin,
		#[error("Score not found")]
		ScoreNotFound,
		#[error("Song not found")]
		SongNotFound,
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

		// External errors
		#[error("HTTP error: {0}")]
		Http(#[from] reqwest::Error),
		#[error("General network error: {0}")]
		NetworkError(#[from] std::io::Error),
		#[error("Internal EtternaOnline server error (HTTP {status_code})")]
		InternalServerError { status_code: u16 },
		#[error("Error while parsing the json sent by the server ({0})")]
		InvalidJson(#[from] serde_json::Error),
		#[error("Sever responded to query with an unrecognized error message ({0})")]
		UnknownApiError(String),
		#[error("Server sent a payload that doesn't match expectations (debug: {0:?})")]
		InvalidDataStructure(String),
		#[error("Server response was empty")]
		EmptyServerResponse
	}
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
