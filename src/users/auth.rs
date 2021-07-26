use chrono::offset::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";
const EXP_TIME: i64 = 60 * 60 * 24; // 1 day

pub static REGISTER_KEY: &str = "0dce349d-0490-4fef-ba0f-f327ee29bc0e";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct Bearer(pub String);

pub fn read_token(bearer: &str) -> Result<String, String> {
    if !bearer.starts_with(BEARER) {
        return Err("Invalid auth".to_string());
    }
    let jwt = bearer.trim_start_matches(BEARER).to_owned();
    let decoded = decode::<Claims>(
        &jwt,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| format!("Unable to parse token: {}", e))?;
    Ok(decoded.claims.sub)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Bearer {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Bearer, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            println!("unable to load auth from header");
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        match read_token(keys[0]) {
            Ok(token) => Outcome::Success(Bearer(token)),
            Err(e) => {
                println!("auth failure: {}", e);
                Outcome::Failure((Status::Unauthorized, ()))
            }
        }
    }
}

pub struct ApiKey(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<ApiKey, ()> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(ApiKey(keys[0].to_string()))
    }
}

pub fn create_jwt(user: &str) -> Result<(String, i64), String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(EXP_TIME))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.to_owned(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map(|t| (t, expiration))
        .map_err(|_| "Error::JWTTokenCreationError".to_owned())
}
