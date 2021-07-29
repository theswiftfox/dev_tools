use uuid::Uuid;

#[get("/uuid")]
pub fn gen_uuid() -> String {
    Uuid::new_v4().to_string()
}
