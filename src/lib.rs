#![allow(clippy::tabs_in_doc_comments)]

/*!
This crate provides an ergonomic wrapper around the v2 API of
[EtternaOnline](https://etternaonline.com), commonly abbreviated "EO". The EO API requires a valid
username and password combination to expose its functions. You will also need an API token called
"client data".

# Usage
For detailed documentation usage, see [`v2::Session`] or [`web::Session`]
*/

// THIS IS MY TODO LIST:
// - Remove thiserror dependency

mod extension_traits;
mod structs;
pub use structs::*;
pub mod v2;
pub mod web;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	// Normal errors
	#[error("User not found")]
	UserNotFound,
	#[error("Username and password combination not found")]
	InvalidLogin,
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
	#[error("No users registered")]
	NoUsersFound,

	// Meta errors
	#[error("Server response was malformed or unsensical")]
	UnexpectedResponse(String),
	#[error("Error while parsing the json sent by the server")]
	InvalidJson(String),
	#[error("Web server is down")]
	ServerIsDown,
	#[error("Some network error")]
	NetworkError(String),
	#[error("Server returned an unknown error")]
	UnknownApiError(String),
	#[error("Server sent a JSON payload that doesn't match expectations")]
	InvalidJsonStructure(Option<Box<dyn std::error::Error>>),
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