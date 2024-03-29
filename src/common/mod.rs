pub mod structs;
use structs::*;

use etterna::*;

use crate::extension_traits::*;
use crate::Error;

pub(crate) fn note_type_from_eo(note_type: &serde_json::Value) -> Result<etterna::NoteType, Error> {
	match note_type.u32_()? {
		1 => Ok(NoteType::Tap),
		2 => Ok(NoteType::HoldHead),
		3 => Ok(NoteType::HoldTail),
		4 => Ok(NoteType::Mine),
		5 => Ok(NoteType::Lift),
		6 => Ok(NoteType::Keysound),
		7 => Ok(NoteType::Fake),
		other => Err(Error::InvalidDataStructure(format!(
			"Unexpected note type integer {}",
			other
		))),
	}
}

pub(crate) fn skillset_to_eo(skillset: etterna::Skillset7) -> &'static str {
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

fn parse_replay_inner(json: &serde_json::Value) -> Result<Option<Replay>, Error> {
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

	let json: serde_json::Value = serde_json::from_str(replay_str)?;

	let notes = json
		.array()?
		.iter()
		.map(|note_json| {
			let note_json = note_json.array()?;

			let (lane_json, tick_json) = if let [_, _, tick_json] = note_json.as_slice() {
				// In many of Bobini's scores (e.g. S0021ccf183b2bcbe716f0b875e321a85f90230b6263),
				// the rows only have three entries, where the third is the tick instead of lane
				(None, Some(tick_json))
			} else {
				// But normally, the third entry is the lane and the fifth entry is the tick
				(note_json.get(2), note_json.get(4))
			};

			Ok(ReplayNote {
				time: note_json[0].f32_()?,
				hit: {
					let deviation = note_json[1].f32_()? / 1000.0;
					if (deviation - 0.18).abs() < 0.0000001 {
						etterna::Hit::Miss
					} else {
						etterna::Hit::Hit { deviation }
					}
				},
				lane: match lane_json {
					Some(json) => {
						json.attempt_get("lane u8, maybe -1", |json| match json.as_i64()? {
							-1 => Some(None),
							lane @ 0..=255 => Some(Some(lane as u8)),
							_ => None, // everything else is invalid
						})?
					}
					None => None,
				},
				note_type: match note_json.get(3) {
					Some(json) => Some(note_type_from_eo(json)?),
					None => None,
				},
				tick: match tick_json {
					// it doesn't exist sometimes like in Sd4fc92514db02424e6b3fe7cdc0c2d7af3cd3dda6526
					Some(x) => Some(x.u32_()?),
					None => None,
				},
			})
		})
		.collect::<Result<Vec<ReplayNote>, Error>>()?;

	// I encountered this on the following Grief & Malice score:
	// https://etternaonline.com/score/view/S0a7d27562ee566ae445ee08fc0b4a182d0ad6cfb3358
	if notes.len() == 0 {
		return Ok(None);
	}

	Ok(Some(Replay { notes }))
}

pub(crate) fn parse_replay(json: &serde_json::Value) -> Option<Replay> {
	match parse_replay_inner(json) {
		Ok(Some(x)) => Some(x),
		Ok(None) => None,
		Err(e) => {
			log::warn!("failed to parse replay: {}", e);
			None
		}
	}
}
