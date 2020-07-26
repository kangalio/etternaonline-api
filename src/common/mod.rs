pub(crate) mod structs;
pub use structs::*;

use crate::Error;
use crate::extension_traits::*;

pub(crate) fn note_type_from_eo(note_type: &serde_json::Value) -> Result<NoteType, Error> {
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

pub(crate) fn skillset_to_eo(skillset: Skillset7) -> &'static str {
	match skillset {
		Skillset7::Stream => "Stream",
		Skillset7::Jumpstream => "Jumpstream",
		Skillset7::Handstream => "Handstream",
		Skillset7::Stamina => "Stamina",
		Skillset7::Jackspeed => "JackSpeed",
		Skillset7::Chordjack => "Chordjack",
		Skillset7::Technical => "Technical",
	}
}

pub(crate) fn parse_replay(json: &serde_json::Value) -> Result<Option<Replay>, Error> {
	if json.is_null() {
		return Ok(None);
	}

	let replay_str = match json {
		serde_json::Value::Array(values) => match values[0].as_str() {
			Some(x) => x,
			None => return Ok(None),
		},
		serde_json::Value::String(string) => string,
		_ => return Ok(None),
	};

	let json: serde_json::Value = serde_json::from_str(replay_str)
		.map_err(|e| Error::InvalidJson(format!("{}", e)))?;

	let notes = json.array()?.iter().map(|note_json| Ok({
		let note_json = note_json.as_array().unwrap();
		ReplayNote {
			time: note_json[0].f32_()?,
			deviation: note_json[1].f32_()? / 1000.0,
			lane: note_json[2].as_i64().unwrap() as u8,
			note_type: note_type_from_eo(&note_json[3])?,
			tick: note_json.get(4).map(|x| x.as_i64().unwrap() as u32), // it doesn't exist sometimes like in Sd4fc92514db02424e6b3fe7cdc0c2d7af3cd3dda6526
		}
	})).collect::<Result<Vec<ReplayNote>, Error>>()?;

	Ok(Some(Replay { notes }))
}