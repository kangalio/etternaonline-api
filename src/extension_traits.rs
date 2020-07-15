use crate::Error;

pub(crate) trait ApiUnwrap<T> {
	fn json_unwrap(self) -> Result<T, Error>;
}

impl<T> ApiUnwrap<T> for Option<T> {
	fn json_unwrap(self) -> Result<T, Error> {
		self.ok_or(Error::InvalidJsonStructure(None))
	}
}

impl<T, E: std::error::Error> ApiUnwrap<T> for Result<T, E> where E: 'static {
	fn json_unwrap(self) -> Result<T, Error> {
		self.map_err(|e| Error::InvalidJsonStructure(Some(Box::new(e))))
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