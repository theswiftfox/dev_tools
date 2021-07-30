use crate::errors::ApiError;
use crate::errors::ApiErrorCodes;
use crate::secrets::{ApiKey, API_KEYS, REGISTER_KEY};
use crate::users::{Credentials, User};
use crate::DbConn;

use rocket::http::Status;
use rocket::serde::json::{Json, Value};

#[post("/register", format = "application/json", data = "<creds>")]
pub async fn register(
    creds: Json<Credentials>,
    conn: DbConn,
    api_key: ApiKey,
) -> Result<Json<User>, ApiError> {
    API_KEYS
        .check_api_key(REGISTER_KEY, &api_key.0)
        .map_err(|_| ApiError {
            code: ApiErrorCodes::Forbidden,
            scope: None,
            message: None,
        })?;

    conn.run(|c| User::create(&creds.into_inner(), c))
        .await
        .map(|u| Json::from(u))
        .map_err(|e| {
            let mut err = ApiError::from(e);
            if err.code == ApiErrorCodes::InvalidField {
                err.scope = Some("username".to_owned());
                err.message = Some("already in use".to_owned());
            }
            err
        })
}

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login(user: Json<Credentials>, conn: DbConn) -> Result<Json<Value>, Status> {
    conn.run(|c| User::login(&user.into_inner(), c)).await
}
