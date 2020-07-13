#[derive(Debug, PartialEq, Clone)]
pub struct UserDetails {
	pub username: String,
	pub about_me: String,
	pub is_moderator: bool,
	pub is_patreon: bool,
	pub avatar_url: String,
	pub country_code: String,
	pub player_rating: f64,
	pub default_modifiers: String,
	pub rating: Skillsets,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skillsets {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64,
}

impl Skillsets {
	pub fn overall(&self) -> f64 {
		(self.stream
			+ self.jumpstream
			+ self.handstream
			+ self.stamina
			+ self.jackspeed
			+ self.chordjack
			+ self.technical)
			/ 7.0
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopScore {
	/// The scorekey, for example `S65565b5bc377c6d78b60c0aecfd9e05955b4cf63`
	pub scorekey: String,
	/// The song name, for example `Game Time`
	pub song_name: String,
	/// The overall score-specific-rating
	pub ssr_overall: f64,
	/// Wifescore of the score, from 0.0 to 100.0
	pub wifescore: f64,
	/// The music rate
	pub rate: f64,
	/// The chart difficulty
	pub difficulty: Difficulty,
	/// The key of the chart, for example `X6ea10eba800cfcbfe462e902da3d3cdfb8d546d9`
	pub chartkey: String,
	/// The MSD of the chart on 1.0x
	pub ssr: Skillsets,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatestScore {
	pub scorekey: String,
	pub song_name: String,
	pub ssr_overall: f64,
	pub wifescore: f64,
	pub rate: f64,
	pub difficulty: Difficulty,

}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Skillset7 {
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Skillset8 {
	Overall,
	Stream,
	Jumpstream,
	Handstream,
	Stamina,
	Jackspeed,
	Chordjack,
	Technical,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Difficulty {
	Beginner, Easy, Medium, Hard, Challenge, Edit
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UserRanksPerSkillset {
	pub overall: u32,
	pub stream: u32,
	pub jumpstream: u32,
	pub handstream: u32,
	pub stamina: u32,
	pub jackspeed: u32,
	pub chordjack: u32,
	pub technical: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopScorePerSkillset {
	pub song_name: String,
	pub rate: f64,
	pub wifescore: f64,
	pub chartkey: String,
	pub scorekey: String,
	pub difficulty: Difficulty,
	pub ssr: Skillsets,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserTopScoresPerSkillset {
	pub overall: Vec<TopScorePerSkillset>,
	pub stream: Vec<TopScorePerSkillset>,
	pub jumpstream: Vec<TopScorePerSkillset>,
	pub handstream: Vec<TopScorePerSkillset>,
	pub stamina: Vec<TopScorePerSkillset>,
	pub jackspeed: Vec<TopScorePerSkillset>,
	pub chordjack: Vec<TopScorePerSkillset>,
	pub technical: Vec<TopScorePerSkillset>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreData {
	pub scorekey: String,
	pub ssr: Skillsets,
	/// 0.0 = 0%, 1.0 = 100%
	pub wifescore: f64,
	pub rate: f64,
	pub max_combo: u32,
	pub is_valid: bool,
	pub has_chord_cohesion: bool,
	pub judgements: Judgements,
	pub replay: Option<Replay>,
	pub user: ScoreDataUser,
	pub song_name: String,
	pub artist: String,
	pub song_id: u32,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Hash)]
pub struct Judgements {
	pub marvelouses: u32,
	pub perfects: u32,
	pub greats: u32,
	pub goods: u32,
	pub bads: u32,
	pub misss: u32,
	pub hit_mines: u32,
	pub held_holds: u32,
	pub let_go_holds: u32,
	pub missed_holds: u32,
}

#[derive(Debug, PartialEq, Clone)]
// TODO: maybe rename this? it's used in multiple places, not just score data
pub struct ScoreDataUser {
	pub username: String,
	pub avatar: String,
	pub country_code: String,
	pub overall_rating: f64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Replay {
	pub notes: Vec<ReplayNote>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReplayNote {
	/// The position of the note inside the chart, in seconds
	pub time: f64,
	/// The offset that the note was hit with, in seconds. A 50ms early hit would be `-0.05`
	pub deviation: f64,
	/// The position of the ntoe inside the chart, in ticks (192nds)
	pub tick: u32,
	/// The lane/column that this note appears on. 0-3 for 4k, 0-5 for 6k
	pub lane: u8,
	/// Type of the note (tap, hold, mine etc.)
	pub note_type: NoteType,
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NoteType {
	Tap = 1,
	HoldHead = 2,
	HoldTail = 3,
	Mine = 4,
	Lift = 5,
	Keysound = 6,
	Fake = 7,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartLeaderboardScore {
	pub scorekey: String,
	pub ssr: Skillsets,
	pub wifescore: f64,
	pub rate: f64,
	pub max_combo: u32,
	pub is_valid: bool,
	pub has_chord_cohesion: bool,
	pub datetime: String,
	pub modifiers: String,
	pub has_replay: bool,
	pub judgements: Judgements,
	pub user: ScoreDataUser,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LeaderboardEntry {
	pub user: ScoreDataUser,
	pub rating: Skillsets,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreGoal {
	pub chartkey: String,
	pub rate: f64,
	pub wifescore: f64,
	pub time_assigned: String,
	pub time_achieved: Option<String>,
}