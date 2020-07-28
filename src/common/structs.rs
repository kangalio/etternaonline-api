#[doc(no_inline)]
pub use etterna::structs::*;

/// Replay data, contains [`ReplayNote`]
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct Replay {
	pub notes: Vec<ReplayNote>,
}

/// A singular note, used inside [`Replay`]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplayNote {
	/// The position of the note inside the chart, in seconds. **Note: EO returns slightly incorrent
	/// values here!**
	pub time: f32,
	/// The offset that the note was hit with, in seconds. A 50ms early hit would be `-0.05`. A miss
	/// is always 0.18
	pub deviation: f32,
	/// The position of the ntoe inside the chart, in ticks (192nds)
	pub tick: Option<u32>,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k
	pub lane: u8,
	/// Type of the note (tap, hold, mine etc.)
	pub note_type: NoteType,
}

impl ReplayNote {
	pub fn is_miss(&self) -> bool {
		(self.deviation - 0.18).abs() < f32::EPSILON
	}
}