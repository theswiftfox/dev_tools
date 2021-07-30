use std::collections::BTreeMap;
use std::fs::read;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

pub struct ApiKeys {
    keys: BTreeMap<String, String>,
}

pub struct ApiKey(pub String);

lazy_static! {
    pub static ref API_KEYS: ApiKeys = ApiKeys::init();
}

pub static REGISTER_KEY: &str = "apikeys.register";

const APIKEY: &str = "ApiKey ";

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<ApiKey, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        let apikey = keys[0];
        if !apikey.starts_with(APIKEY) {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(ApiKey(apikey.trim_start_matches(APIKEY).to_string()))
    }
}

impl ApiKeys {
    fn init() -> ApiKeys {
        let mut keys: BTreeMap<String, String> = BTreeMap::new();
        match read("apikeys.txt") {
            Ok(file_content) => {
                let as_string = String::from_utf8_lossy(&file_content);
                for line in as_string.lines() {
                    let pair: Vec<&str> = line.split("=").collect();
                    if pair.len() == 2 && pair[1].len() > 0 {
                        keys.insert(pair[0].to_string(), pair[1].to_string());
                    } else {
                        eprintln!("Skipped apikey [{}] due to missing value", pair[0]);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Unable to load API-Keys. Protected endpoints will not be supported\n{}",
                    e
                );
            }
        };
        ApiKeys { keys: keys }
    }

    pub fn _read_api_key(&self, node: &str) -> Option<String> {
        match self.keys.get(node) {
            Some(k) => Some(k.clone()),
            None => None,
        }
    }

    pub fn check_api_key(&self, node: &str, value: &str) -> Result<(), ()> {
        match self.keys.get(node) {
            Some(v) => v.eq(value),
            None => false,
        }
        .then(|| (()))
        .ok_or(())
    }
}
