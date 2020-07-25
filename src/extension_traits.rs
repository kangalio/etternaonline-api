use crate::Error;

pub(crate) trait ApiUnwrap<T> {
	fn json_unwrap(self) -> Result<T, Error>;
	fn idk(self, what_was_expected: &'static str, what_we_got: &serde_json::Value) -> Result<T, Error>;
}

impl<T> ApiUnwrap<T> for Option<T> {
	fn json_unwrap(self) -> Result<T, Error> {
		self.ok_or(Error::InvalidJsonStructure(None))
	}

	fn idk(self, what_was_expected: &'static str, what_we_got: &serde_json::Value) -> Result<T, Error> {
		let mut what_we_got = what_we_got.to_string();
		if what_we_got.len() > 100 {
			what_we_got.truncate(100);
			what_we_got += "...";
		}

		let msg = format!("Expected {}, found {}", what_was_expected, what_we_got);
		self.ok_or_else(|| {
			Error::InvalidJsonStructure(Some(msg))
		})
	}
}

impl<T, E: std::error::Error + 'static + Send + Sync> ApiUnwrap<T> for Result<T, E> where E: 'static {
	fn json_unwrap(self) -> Result<T, Error> {
		self.map_err(|e| Error::InvalidJsonStructure(Some(e.to_string())))
	}

	fn idk(self, what_was_expected: &'static str, what_we_got: &serde_json::Value) -> Result<T, Error> {
		self.ok().idk(what_was_expected, what_we_got)
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

	fn str_(&self) -> Result<&str, Error> {
		self.get().as_str()
			.idk("string", self.get())
	}

	fn string(&self) -> Result<String, Error> {
		Ok(self.str_()?.to_owned())
	}

	fn string_maybe(&self) -> Result<Option<String>, Error> {
		(|| Some(if self.get().is_null() {
			None
		} else {
			Some(self.get().as_str()?.to_owned())
		}))()
			.idk("null or a string", self.get())
	}

	fn u32_string(&self) -> Result<u32, Error> {
		(|| Some(self.get().as_str()?.parse::<u32>().ok()?))()
			.idk("u32 in a string", self.get())
	}

	fn array(&self) -> Result<&Vec<serde_json::Value>, Error> {
		self.get().as_array()
			.idk("array", self.get())
	}

	fn bool_int_string(&self) -> Result<bool, Error> {
		(|| match self.get().as_str()? {
			"0" => Some(false),
			"1" => Some(true),
			_ => None,
		})()
			.idk("\"0\" or \"1\"", self.get())
	}

	fn f32_string(&self) -> Result<f32, Error> {
		(|| Some(self.get().as_str()?.parse::<f32>().ok()?))()
			.idk("f32 in a string", self.get())
	}

	fn u64_(&self) -> Result<u64, Error> {
		self.get().as_u64()
			.idk("u64", self.get())
	}

	fn u32_(&self) -> Result<u32, Error> {
		Ok(self.u64_()? as u32)
	}

	fn f32_(&self) -> Result<f32, Error> {
		(|| Some(self.get().as_f64()? as f32))()
			.idk("f32", self.get())
	}

	fn difficulty_string(&self) -> Result<crate::Difficulty, Error> {
		(|| Some(crate::Difficulty::from_long_string(self.get().as_str()?)?))()
			.idk("difficulty", self.get())
	}

	fn singular_array_item(&self) -> Result<&serde_json::Value, Error> {
		(|| {
			let arr = self.get().as_array()?;
			match arr.len() {
				1 => Some(&arr[0]),
				_ => None,
			}
		})()
			.idk("array with a single item", self.get())
	}
}

impl JsonValueExt for serde_json::Value {
	// `self` intensifies
	fn get(&self) -> &Self { self }
}