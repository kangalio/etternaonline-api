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

	fn str_(&self) -> Result<&str, Error> {
		Ok(self.get().as_str().json_unwrap()?)
	}

	fn string(&self) -> Result<String, Error> {
		Ok(self.get().as_str().json_unwrap()?.to_owned())
	}

	fn string_maybe(&self) -> Result<Option<String>, Error> {
		Ok(if self.get().is_null() {
			None
		} else {
			Some(self.string()?)
		})
	}

	fn u32_string(&self) -> Result<u32, Error> {
		Ok(self.get().str_()?.parse().json_unwrap()?)
	}

	fn array(&self) -> Result<&Vec<serde_json::Value>, Error> {
		Ok(self.get().as_array().json_unwrap()?)
	}

	fn bool_int_string(&self) -> Result<bool, Error> {
		Ok(match self.get().str_()? {
			"0" => false,
			"1" => true,
			other => return Err(Error::InvalidJsonStructure(
				Some(format!("Expected '0' or '1', got {}", other))
			)),
		})
	}

	fn f32_string(&self) -> Result<f32, Error> {
		Ok(self.str_()?.parse().json_unwrap()?)
	}
}

impl JsonValueExt for serde_json::Value {
	// `self` intensifies
	fn get(&self) -> &Self { self }
}