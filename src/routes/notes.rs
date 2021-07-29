use crate::notes::{self, DbErr, Note, NoteForInsert, NotesList};
use crate::users::auth::Bearer;
use crate::DbConn;

use rocket::http::Status;
use rocket::serde::json::Json;

use rocket_dyn_templates::Template;

#[get("/note/<id>")]
pub async fn get_note(id: i32, user: Bearer, conn: DbConn) -> Result<Template, Status> {
    let note = conn
        .run(move |c| notes::get_note(id, &user, c))
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Template::render("notes", vec![note]))
}

#[delete("/note/<id>")]
pub async fn delete_note(id: i32, user: Bearer, conn: DbConn) -> Result<Status, Status> {
    conn.run(move |c| notes::delete_note(id, &user, c))
        .await
        .map_err(|e| match e {
            DbErr::Forbidden => Status::Forbidden,
            _ => Status::InternalServerError,
        })?;
    Ok(Status::Ok)
}

#[post("/note", format = "application/json", data = "<note>")]
pub async fn create_note(
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
pub async fn create_empty_note(user: Bearer, conn: DbConn) -> Result<Template, Status> {
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
pub async fn create_note_bulk(
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
pub async fn update_note(user: Bearer, note: Json<Note>, conn: DbConn) -> Result<Status, Status> {
    conn.run(move |c| notes::update(note.into_inner(), &user, c))
        .await
        .map_err(|_| Status::InternalServerError)?;
    Ok(Status::Ok)
}

#[get("/notes")]
pub async fn get_notes(user: Bearer, conn: DbConn) -> Result<Template, Status> {
    let notes = conn
        .run(move |c| notes::get_notes(&user, c))
        .await
        .map(|n| n)
        .map_err(|_| Status::InternalServerError)?;
    Ok(Template::render("notes", &notes))
}
