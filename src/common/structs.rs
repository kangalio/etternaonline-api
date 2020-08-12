#[doc(no_inline)]
pub use etterna::structs::*;

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
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
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
	pub fn split_into_lanes(&self) -> Option<([Vec<f32>; 4], [Vec<f32>; 4])> {
		let mut note_seconds_columns = [vec![], vec![], vec![], vec![]];
		// timing of player hits. EXCLUDING MISSES!!!! THEY ARE NOT PRESENT IN THESE VECTORS!!
		let mut hit_seconds_columns = [vec![], vec![], vec![], vec![]];

		for note in self.notes.iter() {
			if note.lane? >= 4 { continue }

			if !(note.note_type? == etterna::NoteType::Tap || note.note_type? == etterna::NoteType::HoldHead) {
				continue;
			}

			note_seconds_columns[note.lane? as usize].push(note.time);
			if let etterna::Hit::Hit { deviation } = note.hit {
				hit_seconds_columns[note.lane? as usize].push(note.time + deviation);
			}
		}

		Some((note_seconds_columns, hit_seconds_columns))
	}

	/// Like [`Self::split_into_lanes`], but it doesn't split by lane. Instead, everything is put
	/// into one big vector instead.
	/// 
	/// Even non-4k notes are included in this function's result!
	/// 
	/// If this replay doesn't have note type information, None is returned.
	pub fn split_into_notes_and_hits(&self) -> Option<(Vec<f32>, Vec<f32>)> {
		let mut note_seconds = Vec::with_capacity(self.notes.len());
		// timing of player hits. EXCLUDING MISSES!!!! THEY ARE NOT PRESENT IN THESE VECTORS!!
		let mut hit_seconds = Vec::with_capacity(self.notes.len());

		for note in self.notes.iter() {
			if !(note.note_type? == etterna::NoteType::Tap || note.note_type? == etterna::NoteType::HoldHead) {
				continue;
			}

			note_seconds.push(note.time);
			if let etterna::Hit::Hit { deviation } = note.hit {
				hit_seconds.push(note.time + deviation);
			}
		}

		Some((note_seconds, hit_seconds))
	}
}

impl etterna::SimpleReplay for Replay {
	fn iter_hits(&self) -> Box<dyn '_ + Iterator<Item = etterna::Hit>> {
		Box::new(self.notes.iter().map(|note| note.hit))
	}
}

/// A singular note, used inside [`Replay`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_split_replay() {
		let replay = Replay { notes: vec![
			ReplayNote { time: 0.0, hit: etterna::Hit::Hit { deviation: 0.15 }, lane: Some(0), note_type: Some(NoteType::Tap), tick: None},
			ReplayNote { time: 1.0, hit: etterna::Hit::Hit { deviation: -0.03 }, lane: Some(1), note_type: Some(NoteType::Tap), tick: None},
			ReplayNote { time: 2.0, hit: etterna::Hit::Miss, lane: Some(2), note_type: Some(NoteType::Tap), tick: None},
			ReplayNote { time: 3.0, hit: etterna::Hit::Hit { deviation: 0.50 }, lane: Some(3), note_type: Some(NoteType::Tap), tick: None},
			ReplayNote { time: 4.0, hit: etterna::Hit::Hit { deviation: 0.15 }, lane: Some(0), note_type: Some(NoteType::Tap), tick: None},
		] };

		assert_eq!(
			replay.split_into_notes_and_hits(),
			Some((
				vec![0.0, 1.0, 2.0, 3.0, 4.0],
				vec![0.15, 0.97, /* miss omitted */ 3.5, 4.15],
			))
		);

		assert_eq!(
			replay.split_into_lanes(),
			Some((
				[vec![0.0, 4.0], vec![1.0], vec![2.0], vec![3.0]],
				[vec![0.15, 4.15], vec![0.97], vec![], vec![3.5]],
			))
		);

		assert_eq!(
			Replay { notes: vec![] }.split_into_notes_and_hits(),
			Some((
				vec![],
				vec![],
			))
		);
		
		assert_eq!(
			Replay { notes: vec![] }.split_into_lanes(),
			Some((
				[vec![], vec![], vec![], vec![]],
				[vec![], vec![], vec![], vec![]],
			))
		);
	}
}