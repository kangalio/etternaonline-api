use crate::Error;

pub struct Session {
	// Rate limiting stuff
	last_request: std::time::Instant,
	rate_limit: std::time::Duration,

	timeout: Option<std::time::Duration>,
}

impl Session {
	pub fn new_from_login(
		rate_limit: std::time::Duration,
		timeout: Option<std::time::Duration>,
	) -> Self {
		Self {
			rate_limit, timeout,
			last_request: std::time::Instant::now(), // this's not really true but oh well
		}
	}
}