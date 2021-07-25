mod schema;

use crate::users::auth::Bearer;
use diesel::prelude::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use diesel::sqlite::SqliteConnection;
use diesel::{AsChangeset, Insertable, Queryable};
use schema::notes;
use serde::{Deserialize, Serialize};

pub enum DbErr {
    Forbidden,
    UpdateError,
}

#[derive(Serialize, Deserialize, Queryable, AsChangeset, Clone)]
#[table_name = "notes"]
pub struct Note {
    id: i32,
    creator: String,
    title: Option<String>,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct NotesList {
    pub notes: Vec<Note>,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[table_name = "notes"]
pub struct NoteForInsert {
    title: Option<String>,
    creator: String,
    content: String,
}

pub fn create(note: NoteForInsert, user: &Bearer, conn: &SqliteConnection) -> QueryResult<Note> {
    let insert_note = NoteForInsert {
        title: note.title,
        creator: user.0.clone(),
        content: note.content,
    };
    diesel::insert_into(notes::table)
        .values(&insert_note)
        .execute(conn)?;
    notes::table.order(notes::id.desc()).first::<Note>(conn)
}

pub fn update(note: Note, user: &Bearer, conn: &SqliteConnection) -> Result<(), DbErr> {
    match notes::table
        .filter(notes::creator.eq(&user.0))
        .find(note.id)
        .first::<Note>(conn)
    {
        Ok(_) => (),
        Err(_) => return Err(DbErr::Forbidden),
    };
    match diesel::update(notes::table.find(note.id))
        .set(note)
        .execute(conn)
    {
        Ok(_) => Ok(()),
        Err(_) => Err(DbErr::UpdateError),
    }
}

pub fn update_bulk(notes: Vec<Note>, user: &Bearer, conn: &SqliteConnection) -> Result<(), DbErr> {
    for n in notes {
        update(n, user, conn)?
    }
    Ok(())
}

pub fn get_notes(user: &Bearer, conn: &SqliteConnection) -> QueryResult<Vec<Note>> {
    notes::table.filter(notes::creator.eq(&user.0)).load(conn)
}

pub fn delete_note(note: Note, user: &Bearer, conn: &SqliteConnection) {}
