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
#[macro_use] mod structs;
pub use structs::*;
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

fn note_type_from_eo(note_type: &serde_json::Value) -> Result<NoteType, Error> {
	match note_type.as_i64().unwrap() {
		1 => Ok(NoteType::Tap),
		2 => Ok(NoteType::HoldHead),
		3 => Ok(NoteType::HoldTail),
		4 => Ok(NoteType::Mine),
		5 => Ok(NoteType::Lift),
		6 => Ok(NoteType::Keysound),
		7 => Ok(NoteType::Fake),
		other => Err(Error::UnexpectedResponse(format!("Unexpected note type integer {}", other))),
	}
}

fn parse_replay(json: &serde_json::Value) -> Result<Option<Replay>, Error> {
	let replay_str = match json.as_array().unwrap()[0].as_str() {
		Some(replay_str) => replay_str,
		None => return Ok(None),
	};

	let json: serde_json::Value = serde_json::from_str(replay_str)
		.map_err(|e| Error::InvalidJson(format!("{}", e)))?;

	let mut notes = Vec::new();
	for note_json in json.as_array().unwrap() {
		let note_json = note_json.as_array().unwrap();
		notes.push(ReplayNote {
			time: note_json[0].as_f64().unwrap(),
			deviation: note_json[1].as_f64().unwrap() / 1000.0,
			lane: note_json[2].as_i64().unwrap() as u8,
			note_type: note_type_from_eo(&note_json[3])?,
			tick: note_json.get(4).map(|x| x.as_i64().unwrap() as u32), // it doesn't exist sometimes like in Sd4fc92514db02424e6b3fe7cdc0c2d7af3cd3dda6526
		});
	}

	Ok(Some(Replay { notes }))
}