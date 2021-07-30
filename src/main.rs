#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

mod notes;
mod routes;
mod secrets;
mod users;
mod errors;

use rocket::fs::FileServer;
use rocket_dyn_templates::handlebars::{
    Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext,
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::database;

#[database("sqlite_db")]
pub struct DbConn(diesel::SqliteConnection);

fn breakline_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(text) = h.param(0) {
        let string = text.value().render().replace("\n", "<br>");
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
                routes::misc::gen_uuid,
                routes::users::login,
                routes::users::register,
                routes::notes::create_note,
                routes::notes::create_empty_note,
                routes::notes::create_note_bulk,
                routes::notes::get_notes,
                routes::notes::delete_note,
                routes::notes::get_note,
                routes::notes::update_note
            ],
        )
        .mount("/", FileServer::from("static"))
        .attach(DbConn::fairing())
        .attach(Template::custom(|engines| {
            engines
                .handlebars
                .register_helper("breaklines", Box::new(breakline_helper));
        }))
}
