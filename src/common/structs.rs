#[doc(no_inline)]
pub use etterna::structs::*;

/// Replay data, contains [`ReplayNote`]
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
	pub fn split_into_lanes(&self) -> ([Vec<f32>; 4], [Vec<f32>; 4]) {
		let mut note_seconds_columns = [vec![], vec![], vec![], vec![]];
		// timing of player hits. EXCLUDING MISSES!!!! THEY ARE NOT PRESENT IN THESE VECTORS!!
		let mut hit_seconds_columns = [vec![], vec![], vec![], vec![]];

		for hit in self.notes.iter() {
			if hit.lane >= 4 { continue }

			if !(hit.note_type == etterna::NoteType::Tap || hit.note_type == etterna::NoteType::HoldHead) {
				continue;
			}

			note_seconds_columns[hit.lane as usize].push(hit.time);
			if let Some(deviation) = hit.deviation { // if it's not miss
				hit_seconds_columns[hit.lane as usize].push(hit.time + deviation);
			}
		}

		(note_seconds_columns, hit_seconds_columns)
	}

	/// Like [`Self::split_into_lanes`], but it doesn't split by lane. Instead, everything is put
	/// into one big vector instead.
	/// 
	/// Even non-4k notes are included in this function's result!
	pub fn split_into_notes_and_hits(&self) -> (Vec<f32>, Vec<f32>) {
		let mut note_seconds = Vec::with_capacity(self.notes.len());
		// timing of player hits. EXCLUDING MISSES!!!! THEY ARE NOT PRESENT IN THESE VECTORS!!
		let mut hit_seconds = Vec::with_capacity(self.notes.len());

		for hit in self.notes.iter() {
			if !(hit.note_type == etterna::NoteType::Tap || hit.note_type == etterna::NoteType::HoldHead) {
				continue;
			}

			note_seconds.push(hit.time);
			if let Some(deviation) = hit.deviation { // if it's not miss
				hit_seconds.push(hit.time + deviation);
			}
		}

		(note_seconds, hit_seconds)
	}
}

impl etterna::SimpleReplay for Replay {
	fn iter_deviations(&self) -> Box<dyn '_ + Iterator<Item = Option<f32>>> {
		Box::new(self.notes.iter().map(|note| note.deviation))
	}
}

/// A singular note, used inside [`Replay`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplayNote {
	/// The position of the note inside the chart, in seconds. **Note: EO returns slightly incorrect
	/// values here!**
	pub time: f32,
	/// The offset that the note was hit with, in seconds. A 50ms early hit would be `-0.05`. None
	/// if miss
	pub deviation: Option<f32>,
	/// The position of the ntoe inside the chart, in ticks (192nds)
	pub tick: Option<u32>,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k
	pub lane: u8,
	/// Type of the note (tap, hold, mine etc.)
	pub note_type: NoteType,
}

impl ReplayNote {
	pub fn is_miss(&self) -> bool {
		self.deviation.is_none()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_is_miss() {
		let mut dummy_note = ReplayNote {
			time: 0.0,
			deviation: 0.15, // a late bad
			tick: Some(0),
			lane: 3,
			note_type: NoteType::Tap,
		};
		assert!(!dummy_note.is_miss());

		dummy_note.deviation = 0.19;
		assert!(dummy_note.is_miss());
	}

	#[test]
	fn test_split_replay() {
		let replay = Replay { notes: vec![
			ReplayNote { time: 0.0, deviation: 0.15, tick: None, lane: 0, note_type: NoteType::Tap },
			ReplayNote { time: 1.0, deviation: -0.03, tick: None, lane: 1, note_type: NoteType::Tap },
			ReplayNote { time: 2.0, deviation: 0.18, tick: None, lane: 2, note_type: NoteType::Tap },
			ReplayNote { time: 3.0, deviation: 0.50, tick: None, lane: 3, note_type: NoteType::Tap },
			ReplayNote { time: 4.0, deviation: 0.15, tick: None, lane: 0, note_type: NoteType::Tap },
		] };

		assert_eq!(
			replay.split_into_notes_and_hits(),
			(
				vec![0.0, 1.0, 2.0, 3.0, 4.0],
				vec![0.15, 0.97, /* 2x omitted */ 4.15],
			)
		);

		assert_eq!(
			replay.split_into_lanes(),
			(
				[vec![0.0, 4.0], vec![1.0], vec![2.0], vec![3.0]],
				[vec![0.15, 4.15], vec![0.97], vec![], vec![]],
			)
		);
	}
}