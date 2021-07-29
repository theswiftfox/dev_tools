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
) -> Result<Json<User>, Status> {
    API_KEYS
        .check_api_key(REGISTER_KEY, &api_key.0)
        .map_err(|_| Status::Forbidden)?;

    conn.run(|c| User::create(&creds.into_inner(), c))
        .await
        .map(|u| Json::from(u))
        .map_err(|_| Status::InternalServerError)
}

#[post("/login", format = "application/json", data = "<user>")]
pub async fn login(user: Json<Credentials>, conn: DbConn) -> Result<Json<Value>, Status> {
    conn.run(|c| User::login(&user.into_inner(), c)).await
}
