mod structs;
pub use structs::*;

use crate::{Error, extension_traits::*};

pub struct Session {
	// Rate limiting stuff
	last_request: std::time::Instant,
	request_cooldown: std::time::Duration,

	timeout: Option<std::time::Duration>,
}

impl Session {
	pub fn new_from_login(
		request_cooldown: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Self {
		Self {
			request_cooldown, timeout,
			last_request: std::time::Instant::now(), // this's not really true but oh well
		}
	}

	fn request(&mut self,
		method: &str,
		path: &str,
		request_callback: impl Fn(ureq::Request) -> ureq::Response,
	) -> Result<ureq::Response, Error> {
		crate::rate_limit(&mut self.last_request, self.request_cooldown);

		let mut request = ureq::request(method, &format!("https://etternaonline.com/{}", path));
		if let Some(timeout) = self.timeout {
			request.timeout(timeout);
		}
		let response = request_callback(request);

		Ok(response)
	}

	pub fn packlist(&mut self,
		range_to_retrieve: std::ops::Range<u32>
	) -> Result<Vec<PackEntry>, Error> {
		if range_to_retrieve.start >= range_to_retrieve.end {
			return Ok(vec![]);
		}

		let start = range_to_retrieve.start;
		let length = range_to_retrieve.end - range_to_retrieve.start;

		let json = self.request("POST", "pack/packlist", |mut r| r
			.send_form(&[
				("start", &start.to_string()),
				("length", &length.to_string()),
			])
		)?.into_json()?;

		json["data"].as_array().json_unwrap()?.iter()
			.map(|json| Ok(PackEntry {
				average_msd: json["average"].as_str().json_unwrap()?
					.extract("\" />", "</span>").json_unwrap()?
					.parse().json_unwrap()?,
				datetime: json["date"].as_str().json_unwrap()?
					.to_owned(),
				size: json["size"].as_str().json_unwrap()?
					.parse().json_unwrap()?,
				name: json["packname"].as_str().json_unwrap()?
					.extract(">", "</a>").json_unwrap()?
					.to_owned(),
				id: json["packname"].as_str().json_unwrap()?
					.extract("pack/", "\"").json_unwrap()?
					.parse().json_unwrap()?,
				num_votes: json["r_avg"].as_str().json_unwrap()?
					.extract("title='", " votes").json_unwrap()?
					.parse().json_unwrap()?,
				average_vote: json["r_avg"].as_str().json_unwrap()?
					.extract("votes'>", "</div>").json_unwrap()?
					.parse().json_unwrap()?,
				download_link: json["download"].as_str().json_unwrap()?
					.extract("href=\"", "\">").json_unwrap()?
					.to_owned(),
			}))
			.collect()
	}
}