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
use rocket::serde::json::{Json, Value};
use rocket_dyn_templates::handlebars::{
    Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
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

#[get("/note/<id>")]
async fn get_note(id: i32, user: Bearer, conn: DbConn) -> Result<Template, Status> {
    let note = conn
        .run(move |c| notes::get_note(id, &user, c))
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Template::render("notes", vec![note]))
}

#[delete("/note/<id>")]
async fn delete_note(id: i32, user: Bearer, conn: DbConn) -> Result<Status, Status> {
    conn.run(move |c| notes::delete_note(id, &user, c))
        .await
        .map_err(|e| match e {
            DbErr::Forbidden => Status::Forbidden,
            _ => Status::InternalServerError,
        })?;
    Ok(Status::Ok)
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

#[post("/note", rank = 2)]
async fn create_empty_note(user: Bearer, conn: DbConn) -> Result<Template, Status> {
    conn.run(move |c| {
        notes::create(
            NoteForInsert {
                title: None,
                creator: "".to_owned(),
                content: "".to_owned(),
            },
            &user,
            c,
        )
    })
    .await
    .map(|n| Template::render("notes", vec![n]))
    .map_err(|e| {
        println!("{}", e);
        Status::InternalServerError
    })
}

#[put("/notes", format = "application/json", data = "<notes>")]
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

#[put("/note", format = "application/json", data = "<note>")]
async fn update_note(user: Bearer, note: Json<Note>, conn: DbConn) -> Result<Status, Status> {
    conn.run(move |c| notes::update(note.into_inner(), &user, c))
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

fn breakline_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(text) = h.param(0) {
        let text_js = text.value();
        let orig = text_js.to_string();
        let string = orig[1..orig.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\n", "<br>");
        out.write(&string)?;
    }

    Ok(())
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
                create_empty_note,
                create_note_bulk,
                get_notes,
                delete_note,
                get_note,
                update_note
            ],
        )
        .mount("/", FileServer::from("static"))
        // .register("/", catchers![login_form])
        .attach(DbConn::fairing())
        // .attach(Template::fairing())
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("breaklines", Box::new(breakline_helper));
        }))
}
