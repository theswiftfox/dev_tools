#[macro_use]
extern crate rocket;

use rocket::fs::{FileServer, relative};
use uuid::Uuid;

#[get("/uuid")]
fn gen_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/fn", routes![gen_uuid])
        .mount("/", FileServer::from(relative!("static")))
}
