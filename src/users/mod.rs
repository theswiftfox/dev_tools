pub mod auth;
mod schema;

use bcrypt::{hash, verify, DEFAULT_COST};
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use schema::users;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Insertable, AsChangeset, Clone)]
#[table_name = "users"]
pub struct User {
    pub username: String,
    pub pw_hash: String,
}

#[derive(Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn create(creds: &Credentials, conn: &SqliteConnection) -> QueryResult<User> {
        let hash = hash(&creds.password, DEFAULT_COST).unwrap(); // todo
        let user = User {
            username: creds.username.clone(),
            pw_hash: hash,
        };
        diesel::insert_into(users::table)
            .values(&user)
            .execute(conn)?;
        Ok(user)
    }

    pub fn login(creds: &Credentials, conn: &SqliteConnection) -> Result<Json<Value>, Status> {
        let dbusers = match users::table.find(&creds.username).load::<User>(conn) {
            Ok(u) => u,
            Err(_) => return Err(Status::BadRequest),
        };
        let dbuser = dbusers.first().unwrap();
        match verify(&creds.password, &dbuser.pw_hash).unwrap() {
            true => auth::create_jwt(&creds.username)
                .map(|message| {
                    Json(json!({"success": true, "token": message.0, "exp": message.1 }))
                })
                .map_err(|_| Status::InternalServerError),
            false => Err(Status::BadRequest),
        }
    }
}
