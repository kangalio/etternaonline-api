use etterna::prelude::*;

/// Replay data, contains [`ReplayNote`]
///
/// Some replays don't have tick information. Some replays have neither tick nor note type
/// information. Some replays have neither tick nor note type nor lane information.
///
/// There _are_ some guarantees (judging after expirementation with EO):
/// - If one replay note has a certain piece of data, all other replay notes in that replay will
///   will also have that piece of data.
/// - If a replay has note type information, it will definitely also have lane information. <br/>
///   If a replay has tick information, it will definitely also have both note type and lane
///   information.
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(
	feature = "serde",
	serde(crate = "serde_"),
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct Replay {
	pub notes: Vec<ReplayNote>,
}

impl Replay {
	/// Splits the replay into arrays that contain the note and hit seconds, respectively. Note: if
	/// a note was missed, it has no entry in the hit seconds vector - logically, because there
	/// _was_ no hit, hence the miss. A consequence of this is that the nth note second array will
	/// probably not have the same length as the nth hit second array.
	///
	/// Also, this function will discard anything not related to straight tapping, that is, mines,
	/// lifts... Also, everything above 4k will be discarded as well.
	///
	/// If this replay file adheres to the usual Etterna replay ordering, the second lists (hits)
	/// will be sorted ascendingly.
	///
	/// If this replay doesn't have lane and note_type information, None is returned.
	pub fn split_into_lanes(&self) -> Option<[NoteAndHitSeconds; 4]> {
		let mut lanes = [
			NoteAndHitSeconds {
				note_seconds: vec![],
				hit_seconds: vec![],
			},
			NoteAndHitSeconds {
				note_seconds: vec![],
				hit_seconds: vec![],
			},
			NoteAndHitSeconds {
				note_seconds: vec![],
				hit_seconds: vec![],
			},
			NoteAndHitSeconds {
				note_seconds: vec![],
				hit_seconds: vec![],
			},
		];

		for note in self.notes.iter() {
			if note.lane? >= 4 {
				continue;
			}

			if !(note.note_type? == etterna::NoteType::Tap
				|| note.note_type? == etterna::NoteType::HoldHead)
			{
				continue;
			}

			lanes[note.lane? as usize].note_seconds.push(note.time);
			if let etterna::Hit::Hit { deviation } = note.hit {
				lanes[note.lane? as usize]
					.hit_seconds
					.push(note.time + deviation);
			}
		}

		Some(lanes)
	}

	/// Like [`Self::split_into_lanes`], but it doesn't split by lane. Instead, everything is put
	/// into one big vector instead.
	///
	/// Even non-4k notes are included in this function's result!
	///
	/// If this replay doesn't have note type information, None is returned.
	pub fn split_into_notes_and_hits(&self) -> Option<NoteAndHitSeconds> {
		let mut result = NoteAndHitSeconds {
			note_seconds: Vec::with_capacity(self.notes.len()),
			hit_seconds: Vec::with_capacity(self.notes.len()),
		};

		for note in self.notes.iter() {
			if !(note.note_type? == etterna::NoteType::Tap
				|| note.note_type? == etterna::NoteType::HoldHead)
			{
				continue;
			}

			result.note_seconds.push(note.time);
			if let etterna::Hit::Hit { deviation } = note.hit {
				result.hit_seconds.push(note.time + deviation);
			}
		}

		Some(result)
	}
}

impl etterna::SimpleReplay for Replay {
	fn iter_hits(&self) -> Box<dyn '_ + Iterator<Item = etterna::Hit>> {
		Box::new(self.notes.iter().map(|note| note.hit))
	}
}

/// A singular note, used inside [`Replay`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
	feature = "serde",
	serde(crate = "serde_"),
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct ReplayNote {
	/// The position of the note inside the chart, in seconds. **Note: EO returns slightly incorrect
	/// values here!**
	pub time: f32,
	/// The offset that the note was hit with
	pub hit: etterna::Hit,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k. None if not provided by
	/// EO
	pub lane: Option<u8>,
	/// Type of the note (tap, hold, mine etc.). None if not provided by EO
	pub note_type: Option<NoteType>,
	/// The position of the note inside the chart, in ticks (192nds). None if not provided by EO
	pub tick: Option<u32>,
}

/// Represents a file size
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
#[cfg_attr(
	feature = "serde",
	serde(crate = "serde_"),
	derive(serde::Serialize, serde::Deserialize)
)]
pub struct FileSize {
	bytes: u64,
}

impl FileSize {
	/// Create a new file size from the given number of bytes
	pub fn from_bytes(bytes: u64) -> Self {
		Self { bytes }
	}

	/// Get the number of bytes
	pub fn bytes(self) -> u64 {
		self.bytes
	}

	/// Get the number of kilobytes, rounded down
	pub fn kb(self) -> u64 {
		self.bytes / 1_000
	}

	/// Get the number of megabytes, rounded down
	pub fn mb(self) -> u64 {
		self.bytes / 1_000_000
	}

	/// Get the number of gigabytes, rounded down
	pub fn gb(self) -> u64 {
		self.bytes / 1_000_000_000
	}

	/// Get the number of terabytes, rounded down
	pub fn tb(self) -> u64 {
		self.bytes / 1_000_000_000_000
	}
}

thiserror_lite::err_enum! {
	/// Error returned from `FileSize::from_str`
	#[derive(Debug)]
	pub enum FileSizeParseError {
		#[error("Given string was empty")]
		EmptyString,
		#[error("Error while parsing the filesize number")]
		InvalidNumber(#[from] std::num::ParseFloatError),
		#[error("No KB/MB/... ending")]
		NoEnding,
		#[error("Unknown ending (the KB/MB/... thingy)")]
		UnexpectedEnding(String),
	}
}

impl std::str::FromStr for FileSize {
	type Err = FileSizeParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut token_iter = s.split_whitespace();
		let number: f64 = token_iter
			.next()
			.ok_or(FileSizeParseError::EmptyString)?
			.parse()
			.map_err(FileSizeParseError::InvalidNumber)?;
		let ending = token_iter.next().ok_or(FileSizeParseError::NoEnding)?;

		let ending = ending.to_lowercase();
		let multiplier: u64 = match &ending as &str {
			"b" => 1,
			"kb" => 1000,
			"kib" => 1024,
			"mb" => 1000 * 1000,
			"mib" => 1024 * 1024,
			"gb" => 1000 * 1000 * 1000,
			"gib" => 1024 * 1024 * 1024,
			"tb" => 1000 * 1000 * 1000 * 1000,
			"tib" => 1024 * 1024 * 1024 * 1024,
			_ => return Err(FileSizeParseError::UnexpectedEnding(ending)),
		};

		Ok(Self::from_bytes((number * multiplier as f64) as u64))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_split_replay() {
		let replay = Replay {
			notes: vec![
				ReplayNote {
					time: 0.0,
					hit: etterna::Hit::Hit { deviation: 0.15 },
					lane: Some(0),
					note_type: Some(NoteType::Tap),
					tick: None,
				},
				ReplayNote {
					time: 1.0,
					hit: etterna::Hit::Hit { deviation: -0.03 },
					lane: Some(1),
					note_type: Some(NoteType::Tap),
					tick: None,
				},
				ReplayNote {
					time: 2.0,
					hit: etterna::Hit::Miss,
					lane: Some(2),
					note_type: Some(NoteType::Tap),
					tick: None,
				},
				ReplayNote {
					time: 3.0,
					hit: etterna::Hit::Hit { deviation: 0.50 },
					lane: Some(3),
					note_type: Some(NoteType::Tap),
					tick: None,
				},
				ReplayNote {
					time: 4.0,
					hit: etterna::Hit::Hit { deviation: 0.15 },
					lane: Some(0),
					note_type: Some(NoteType::Tap),
					tick: None,
				},
			],
		};

		assert_eq!(
			replay.split_into_notes_and_hits(),
			Some(NoteAndHitSeconds {
				note_seconds: vec![0.0, 1.0, 2.0, 3.0, 4.0],
				hit_seconds: vec![0.15, 0.97, /* miss omitted */ 3.5, 4.15],
			})
		);

		assert_eq!(
			replay.split_into_lanes(),
			Some([
				NoteAndHitSeconds {
					note_seconds: vec![0.0, 4.0],
					hit_seconds: vec![0.15, 4.15],
				},
				NoteAndHitSeconds {
					note_seconds: vec![1.0],
					hit_seconds: vec![0.97],
				},
				NoteAndHitSeconds {
					note_seconds: vec![2.0],
					hit_seconds: vec![],
				},
				NoteAndHitSeconds {
					note_seconds: vec![3.0],
					hit_seconds: vec![3.5],
				},
			])
		);

		assert_eq!(
			Replay { notes: vec![] }.split_into_notes_and_hits(),
			Some(NoteAndHitSeconds {
				note_seconds: vec![],
				hit_seconds: vec![],
			})
		);

		assert_eq!(
			Replay { notes: vec![] }.split_into_lanes(),
			Some([
				NoteAndHitSeconds {
					note_seconds: vec![],
					hit_seconds: vec![]
				},
				NoteAndHitSeconds {
					note_seconds: vec![],
					hit_seconds: vec![]
				},
				NoteAndHitSeconds {
					note_seconds: vec![],
					hit_seconds: vec![]
				},
				NoteAndHitSeconds {
					note_seconds: vec![],
					hit_seconds: vec![]
				},
			])
		);
	}
}
