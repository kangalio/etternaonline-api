use std::convert::TryInto;

use crate::Error;

pub(crate) trait ApiUnwrap<T> {
	fn json_unwrap(self) -> Result<T, Error>;
}
impl<T> ApiUnwrap<T> for Option<T> {
	fn json_unwrap(self) -> Result<T, Error> {
		self.ok_or(Error::InvalidJsonStructure(None))
	}
}
impl<T, E: std::error::Error + 'static + Send + Sync> ApiUnwrap<T> for Result<T, E> where E: 'static {
	fn json_unwrap(self) -> Result<T, Error> {
		self.map_err(|e| Error::InvalidJsonStructure(Some(e.to_string())))
	}
}

pub(crate) trait ExtractStr {
	fn extract<'a>(&'a self, before: &str, after: &str) -> Option<&'a str>;
}
impl ExtractStr for &str {
	fn extract<'a>(&'a self, before: &str, after: &str) -> Option<&'a str> {
		let start_index = self.find(before)? + before.len();
		let end_index = start_index + self[start_index..].find(after)?;
		Some(&self[start_index..end_index])
	}
}

pub(crate) trait JsonValueExt: Sized {
	fn get(&self) -> &serde_json::Value;

	fn attempt_get<'val, 'content: 'val, T: 'content>(&'val self,
		what_is_expected: &str,
		action: impl FnOnce(&'val serde_json::Value) -> Option<T>
	) -> Result<T, Error> {
		match action(self.get()) {
			Some(result) => Ok(result),
			None => Err(Error::InvalidJsonStructure(Some({
				let mut msg = format!("Expected {}, found {}", what_is_expected, self.get());
				if msg.len() > 500 {
					msg.truncate(500);
					msg += "...";
				}
				msg
			})))
		}
	}

	fn str_(&self) -> Result<&str, Error> {
		self.attempt_get("string", |j| j.as_str())
	}

	fn string(&self) -> Result<String, Error> {
		Ok(self.str_()?.to_owned())
	}

	fn string_maybe(&self) -> Result<Option<String>, Error> {
		self.attempt_get("null or a string", |j| Some(if j.is_null() {
			None
		} else {
			Some(j.as_str()?.to_owned())
		}))
	}

	fn array(&self) -> Result<&Vec<serde_json::Value>, Error> {
		self.attempt_get("array", |j| j.as_array())
	}

	fn bool_(&self) -> Result<bool, Error> {
		self.attempt_get("boolean", |j| j.as_bool())
	}

	fn bool_int(&self) -> Result<bool, Error> {
		self.attempt_get("0 or 1", |j| match j.as_i64()? {
			0 => Some(false),
			1 => Some(true),
			_ => None,
		})
	}

	fn bool_int_string(&self) -> Result<bool, Error> {
		self.attempt_get("\"0\" or \"1\"", |j| match j.as_str()? {
			"0" => Some(false),
			"1" => Some(true),
			_ => None,
		})
	}

	fn parse<T: std::str::FromStr>(&self) -> Result<T, Error> {
		let what_were_retrieving = format!("{} in a string", std::any::type_name::<T>());
		let how_were_retrieving_it = |j: &serde_json::Value| j.as_str()?.parse::<T>().ok();

		self.attempt_get(&what_were_retrieving, how_were_retrieving_it)
	}

	fn u64_(&self) -> Result<u64, Error> {
		self.attempt_get("u64", |j| j.as_u64())
	}

	fn i32_(&self) -> Result<i32, Error> {
		self.attempt_get("i32", |j| Some(j.as_i64()?.try_into().ok()?))
	}

	fn u32_(&self) -> Result<u32, Error> {
		self.attempt_get("u32", |j| Some(j.as_u64()?.try_into().ok()?))
	}

	fn f32_(&self) -> Result<f32, Error> {
		self.attempt_get("f32", |j| Some(j.as_f64()? as f32))
	}

	fn singular_array_item(&self) -> Result<&serde_json::Value, Error> {
		self.attempt_get("array with a single item", |j| {
			let arr = j.as_array()?;
			match arr.len() {
				1 => Some(&arr[0]),
				_ => None,
			}
		})
	}

	fn rate_float(&self) -> Result<etterna::Rate, Error> {
		self.attempt_get("rate float", |j| etterna::Rate::from_f32(j.as_f64()? as f32))
	}

	fn wifescore_percent_float(&self) -> Result<etterna::Wifescore, Error> {
		self.attempt_get("wifescore percent float", |j| etterna::Wifescore::from_percent(j.as_f64()? as f32))
	}

	fn wifescore_proportion_float(&self) -> Result<etterna::Wifescore, Error> {
		self.attempt_get("wifescore proportion float", |j| etterna::Wifescore::from_proportion(j.as_f64()? as f32))
	}
	
	fn wifescore_proportion_string(&self) -> Result<etterna::Wifescore, Error> {
		self.attempt_get("wifescore proportion string", |j| etterna::Wifescore::from_proportion(j.as_str()?.parse().ok()?))
	}
}

impl JsonValueExt for serde_json::Value {
	fn get(&self) -> &Self { self } // self intensifies
}