#![allow(clippy::tabs_in_doc_comments, clippy::match_bool)]

/*!
This crate provides an ergonomic wrapper around the v1, v2 and web API of
[EtternaOnline](https://etternaonline.com), commonly abbreviated "EO" (web API is work-in-progress).

Depending on which API you choose, you might need an API token.

# Notes
Etterna terminology:
- The calculated difficulty for a chart is called MSD: Mina standardized difficulty.
- The score rating - which is variable depending on your wifescore - is called SSR:
  score-specific-rating

The wifescores in this library are scaled to a maximum of `1.0`. This is means that a wifescore
of 100% corresponds to a value of `1.0` (as opposed to `100.0`).

# Usage
For detailed usage documentation, see [`v1::Session`] and [`v2::Session`]
*/

// THIS IS MY TODO LIST:
// - Remove thiserror dependency

mod extension_traits;
#[macro_use] mod common;
pub use common::structs::*;
pub mod v1;
pub mod v2;
pub mod web;

use thiserror::Error;

#[cfg(all(feature = "serde", not(feature = "serde_support")))]
compile_error!("Use the `serde_support` feature flag instead of `serde`");

#[derive(Error, Debug, PartialEq, Eq, Clone)]
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
	#[error("Server response was malformed or nonsensical ({0})")]
	UnexpectedResponse(String),
	#[error("Error while parsing the json sent by the server ({0})")]
	InvalidJson(String),
	#[error("Web server is down")]
	ServerIsDown,
	#[error("Network error ({0})")]
	NetworkError(String),
	#[error("Server returned an unknown error ({0})")]
	UnknownApiError(String),
	#[error("Server sent a JSON payload that doesn't match expectations (debug: {0:?})")]
	InvalidJsonStructure(Option<String>),
	#[error("Server timed out")]
	Timeout,
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

// This only works with 4k replays at the moment! All notes beyond the first four columns are
// discarded
pub fn rescore<S, W>(
	replay: &Replay,
	num_hit_mines: u32,
	num_dropped_holds: u32,
	judge: &etterna::Judge,
) -> etterna::Wifescore
where
	S: etterna::ScoringSystem,
	W: etterna::Wife,
{
	let (mut note_seconds_columns, mut hit_seconds_columns) = replay.split_into_lanes();

	let sort = |slice: &mut [f32]| slice.sort_by(|a, b| a.partial_cmp(b).unwrap());
	for column in &mut note_seconds_columns { sort(column); }
	for column in &mut hit_seconds_columns { sort(column); }

	etterna::rescore::<S, W>(
		&note_seconds_columns,
		&hit_seconds_columns,
		num_hit_mines,
		num_dropped_holds,
		judge,
	)
}