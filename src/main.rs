#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

mod notes;
mod users;

use crate::notes::*;
use crate::users::auth::{ApiKey, Bearer, REGISTER_KEY};
use crate::users::{Credentials, User};

use rocket::fs::FileServer;
use rocket::http::Status;
// use rocket::response::Redirect;
use rocket::serde::json::{Json, Value};
// use rocket::Request;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::database;

use uuid::Uuid;

#[database("sqlite_db")]
struct DbConn(diesel::SqliteConnection);

#[get("/uuid")]
fn gen_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[post("/register", format = "application/json", data = "<creds>")]
async fn register(
    creds: Json<Credentials>,
    conn: DbConn,
    api_key: ApiKey,
) -> Result<Json<User>, Status> {
    if api_key.0 != REGISTER_KEY {
        return Err(Status::Forbidden);
    }
    conn.run(|c| users::User::create(&creds.into_inner(), c))
        .await
        .map(|u| Json::from(u))
        .map_err(|_| Status::InternalServerError)
}

#[post("/login", format = "application/json", data = "<user>")]
async fn login(user: Json<Credentials>, conn: DbConn) -> Result<Json<Value>, Status> {
    conn.run(|c| users::User::login(&user.into_inner(), c))
        .await
}

// #[catch(401)]
// fn login_form(_req: &Request) -> Redirect {
//     Redirect::to("/login.html".to_owned())
// }

#[post("/note", format = "application/json", data = "<note>")]
async fn create_note(
    user: Bearer,
    note: Json<NoteForInsert>,
    conn: DbConn,
) -> Result<Json<Note>, Status> {
    conn.run(move |c| notes::create(note.into_inner(), &user, c))
        .await
        .map(|n| Json::from(n))
        .map_err(|e| {
            println!("{}", e);
            Status::InternalServerError
        })
}

#[post("/notes", format = "application/json", data = "<notes>")]
async fn create_note_bulk(
    user: Bearer,
    notes: Json<NotesList>,
    conn: DbConn,
) -> Result<Status, Status> {
    conn.run(move |c| notes::update_bulk(notes.into_inner().notes, &user, c))
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Status::Ok)
}

#[get("/notes")]
async fn get_notes(user: Bearer, conn: DbConn) -> Result<Template, Status> {
    let notes = conn
        .run(move |c| notes::get_notes(&user, c))
        .await
        .map(|n| n)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Template::render("notes", &notes))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/fn",
            routes![
                gen_uuid,
                login,
                register,
                create_note,
                get_notes,
                create_note_bulk
            ],
        )
        .mount("/", FileServer::from("static"))
        // .register("/", catchers![login_form])
        .attach(DbConn::fairing())
        .attach(Template::fairing())
}
