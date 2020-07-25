#![allow(clippy::tabs_in_doc_comments)]

/*!
This crate provides an ergonomic wrapper around the v1 and v2 API of
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

#[derive(Error, Debug, PartialEq, Eq, Clone)]
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
	#[error("Server response was malformed or nonsensical")]
	UnexpectedResponse(String),
	#[error("Error while parsing the json sent by the server")]
	InvalidJson(String),
	#[error("Web server is down")]
	ServerIsDown,
	#[error("Some network error")]
	NetworkError(String),
	#[error("Server returned an unknown error")]
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

// this needs to be here for some reason, and it also needs to be publically accessible because MACROS
#[doc(hidden)]
#[macro_export]
macro_rules! impl_get8 {
	($struct_type:ty, $return_type:ty, $self_:ident, $overall_getter:expr) => {
		impl $struct_type {
			pub fn get(&self, skillset: impl Into<Skillset8>) -> $return_type {
				let $self_ = self;
				match skillset.into() {
					Skillset8::Overall => $overall_getter,
					Skillset8::Stream => self.stream,
					Skillset8::Jumpstream => self.jumpstream,
					Skillset8::Handstream => self.handstream,
					Skillset8::Stamina => self.stamina,
					Skillset8::Jackspeed => self.jackspeed,
					Skillset8::Chordjack => self.chordjack,
					Skillset8::Technical => self.technical,
				}
			}
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