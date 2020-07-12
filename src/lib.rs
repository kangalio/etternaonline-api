use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
	#[error("User not found")]
	UserNotFound,
	#[error("Username and password combination not found")]
	InvalidLogin,
	#[error("Server returned a response code that was not expected")]
	UnexpectedResponseCode(u16),
	#[error("Error while parsing the json sent by the server")]
	InvalidJson(String),
}

pub struct User {
	pub username: String,
	pub about_me: String,
	pub is_moderator: bool,
	pub is_patreon: bool,
	pub avatar_url: String,
	pub country_code: String,
	pub player_rating: f64,
	pub default_modifiers: String,
	pub skillsets: Skillsets,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Skillsets {
	pub stream: f64,
	pub jumpstream: f64,
	pub handstream: f64,
	pub stamina: f64,
	pub jackspeed: f64,
	pub chordjack: f64,
	pub technical: f64
}

pub struct Session {
	// This stuff is needed for re-login
	username: String,
	password: String,
	client_data: String,

    // The auth key that we get from the server on login
    authorization: String,
    
    // Rate limiting stuff
    last_request: std::time::Instant,
    rate_limit: std::time::Duration,
}

impl Session {
    /// Initiate a new session by logging in using the specified credentials and API token.
    /// 
    /// Rate-limiting is done by waiting at least `rate_limit` inbetween requests
	pub fn new_from_login(
		username: String,
		password: String,
        client_data: String,
        rate_limit: std::time::Duration,
	) -> Result<Self, Error> {
		let authorization = "dummy key that will be replaced anyway when I login".into();

		let mut session = Self {
            username, password, client_data, authorization, rate_limit,
            last_request: std::time::Instant::now(),
        };
		session.login()?;

		Ok(session)
	}

    // login again to generate a new session token
	fn login(&mut self) -> Result<(), Error> {
		let response = ureq::post("https://api.etternaonline.com/v2/login")
			.send_form(&[
				("username", &self.username),
				("password", &self.password),
				("clientData", &self.client_data)
            ]);

		match response.status() {
			404 => Err(Error::InvalidLogin),
			200 => {
                let json = response.into_json()
                    .map_err(|e| Error::InvalidJson(format!("{}", e)))?;
				let key = json["data"]["attributes"]["accessToken"].as_str()
					.expect("Received an access token that is not a string");
				self.authorization = format!("Bearer {}", key);
				Ok(())
			}
			other => panic!("Unexpected response code {}", other),
		}
	}

	fn request(&mut self,
		builder: impl Fn() -> ureq::Request
	) -> Result<(u16, serde_json::Value), Error> {

        // Do tha rate-limiting
        let time_since_last_request = std::time::Instant::now().duration_since(self.last_request);
        if time_since_last_request < self.rate_limit {
            std::thread::sleep(self.rate_limit - time_since_last_request);
        }
        self.last_request = std::time::Instant::now();

		let response = builder()
			.set("Authorization", &self.authorization)
			.call();
		
		if response.status() == 401 {
			// Token expired, let's login again and retry
			self.login()?;
			self.request(builder)
		} else {
            let status = response.status();
            let json = response.into_json()
                .map_err(|e| Error::InvalidJson(format!("{}", e)))?;
			Ok((status, json))
		}
	}

    /// Retrieves details about the profile of the specified user.
    /// 
    /// If there is no user with the specified username, `Err(Error::UserNotFound)` is returned.
	pub fn user_details(&mut self, username: &str) -> Result<User, Error> {
		let json = match self.request(|| {
			ureq::get(&format!("https://api.etternaonline.com/v2/user/{}", username))
		})? {
			(200, json) => json,
			(404, _) => return Err(Error::UserNotFound),
			(code, _) => return Err(Error::UnexpectedResponseCode(code)),
		};

		let json = &json["data"]["attributes"];

		Ok(User {
			username: json["userName"].as_str().unwrap().to_owned(),
			about_me: json["aboutMe"].as_str().unwrap().to_owned(),
			is_moderator: json["moderator"].as_bool().unwrap(),
			is_patreon: json["patreon"].as_bool().unwrap(),
			avatar_url: json["avatar"].as_str().unwrap().to_owned(),
			country_code: json["countryCode"].as_str().unwrap().to_owned(),
			player_rating: json["playerRating"].as_f64().unwrap(),
			default_modifiers: json["defaultModifiers"].as_str().unwrap().to_owned(),
			skillsets: Skillsets {
				stream: json["skillsets"]["Stream"].as_f64().unwrap(),
				jumpstream: json["skillsets"]["Jumpstream"].as_f64().unwrap(),
				handstream: json["skillsets"]["Handstream"].as_f64().unwrap(),
				stamina: json["skillsets"]["Stamina"].as_f64().unwrap(),
				jackspeed: json["skillsets"]["JackSpeed"].as_f64().unwrap(),
				chordjack: json["skillsets"]["Chordjack"].as_f64().unwrap(),
				technical: json["skillsets"]["Technical"].as_f64().unwrap(),
			}
		})
	}
}