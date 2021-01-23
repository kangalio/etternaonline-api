#![allow(
	clippy::len_zero,
	clippy::tabs_in_doc_comments,
	clippy::collapsible_if,
	clippy::needless_bool,
	clippy::too_many_arguments
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

thiserror_lite::err_enum! {
	#[derive(Debug)]
	#[non_exhaustive]
	pub enum Error {
		// Normal errors
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

		// Meta errors
		#[error("General network error ({0})")]
		NetworkError(String),
		#[error("Internal web server error (HTTP {status_code})")]
		ServerIsDown { status_code: u16 },
		#[error("Error while parsing the json sent by the server ({0})")]
		InvalidJson(#[from] serde_json::Error),
		#[error("Sever responded to query with an unrecognized error message ({0})")]
		UnknownApiError(String),
		#[error("Server sent a payload that doesn't match expectations (debug: {0:?})")]
		InvalidDataStructure(String),
		#[error("Server timed out")]
		Timeout,
		#[error("Server response was empty")]
		EmptyServerResponse
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Self {
		if e.kind() == std::io::ErrorKind::TimedOut {
			Self::Timeout
		} else {
			Self::NetworkError(e.to_string())
		}
	}
}

fn rate_limit(last_request: &mut std::time::Instant, request_cooldown: std::time::Duration) {
	let now = std::time::Instant::now();
	let time_since_last_request = now.duration_since(*last_request);
	if time_since_last_request < request_cooldown {
		std::thread::sleep(request_cooldown - time_since_last_request);
	}
	*last_request = now;
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
