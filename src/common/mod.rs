pub mod structs;
use structs::*;

use crate::Error;
use crate::extension_traits::*;

pub(crate) fn note_type_from_eo(note_type: &serde_json::Value) -> Result<etterna::NoteType, Error> {
	match note_type.u32_()? {
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

	let json: serde_json::Value = serde_json::from_str(replay_str)?;
	
	// println!("{}", serde_json::to_string_pretty(&json).unwrap());

	let notes = json.array()?.iter().map(|note_json| Ok({
		let note_json = note_json.array()?;
		// println!("{:?}", note_json);
		ReplayNote {
			time: note_json[0].f32_()?,
			deviation: {
				let deviation = note_json[1].f32_()? / 1000.0;
				if (deviation - 0.18).abs() < 0.0000001 {
					None
				} else {
					Some(deviation)
				}
			},
			lane: match note_json.get(2) {
				Some(json) => json.attempt_get("lane u8, maybe -1", |json| match json.as_i64()? {
					-1 => Some(None),
					lane @ 0..=255 => Some(Some(lane as u8)),
					lane => None, // everything else is invalid
				})?,
				None => None,
			},
			note_type: match note_json.get(3) {
				Some(json) => Some(note_type_from_eo(json)?),
				None => None,
			},
			tick: match note_json.get(4) { // it doesn't exist sometimes like in Sd4fc92514db02424e6b3fe7cdc0c2d7af3cd3dda6526
				Some(x) => Some(x.u32_()?),
				None => None,
			},
		}
	})).collect::<Result<Vec<ReplayNote>, Error>>()?;

	// I encountered this on the following Grief & Malice score:
	// https://etternaonline.com/score/view/S0a7d27562ee566ae445ee08fc0b4a182d0ad6cfb3358
	if notes.len() == 0 {
		return Ok(None);
	}

	Ok(Some(Replay { notes }))
}

fn try_lock_immutable<T>(lock: &std::sync::RwLock<T>) -> Option<std::sync::RwLockReadGuard<T>> {
	match lock.try_read() {
		Ok(guard) => Some(guard),
		Err(std::sync::TryLockError::WouldBlock) => None,
		Err(std::sync::TryLockError::Poisoned(poison_err)) => panic!("Poison {:?}", poison_err),
	}
}

pub(crate) struct AuthorizationManager<T> {
    lock: std::sync::RwLock<T>,
    refresh_checking_mutex: std::sync::Mutex<()>,
}

impl<T> AuthorizationManager<T> {
    pub fn new(initial_authorization: T) -> Self {
        Self {
            lock: std::sync::RwLock::new(initial_authorization),
            refresh_checking_mutex: std::sync::Mutex::new(()),
        }
    }
    
    /// The passed closure should perform the actual login request. It _should
    /// not_ simply return the captured login result.
    /// 
    /// If another thread is refreshing right now too, this function will simply
    /// wait until the other thread is finished and then return without having
    /// called the closure.
    pub fn refresh<E>(&self, f: impl FnOnce() -> Result<T, E>) -> Result<(), E> {
        // We lock until we've decided how to proceed (login? wait for other
        // thread?), so that other refresh calls can't interfere
        let refresh_guard = self.refresh_checking_mutex.lock().unwrap();
    
        if try_lock_immutable(&self.lock).is_some() {
            // If we can lock immutably, that means that at max there'll be
            // get_authorization calls active right now
            
            // So let's wait (block) for those calls to finish, and then login
            // and insert the new authoriziation token
            drop(refresh_guard);
            let mut write_guard = self.lock.write().unwrap();
            *write_guard = (f)()?;
        } else {
            // If we can't lock immutably, another thread is logging in right
            // now. So let's wait for them and then just return - there's
            // nothing to do anymore because the other thread has done our login
            // work
			drop(self.lock.read().unwrap());
			
            drop(refresh_guard);
        }
        
        Ok(())
    }
    
    /// Please drop the returned smart pointer as early as possible. The longer
    /// you hold on to it, the longer you block other threads from logging in.
    pub fn get_authorization(&self) -> impl '_ + std::ops::Deref<Target = T> {
        // This will block if a mutable lock is active, i.e. another thread
        // is logging in right now. So we will wait until the other thread
        // finished logging in to get its new fresh authorization value
        let guard = self.lock.read().unwrap();
        
        guard
    }
}