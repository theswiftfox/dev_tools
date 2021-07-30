use std::io::Cursor;

use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error as DieselError;
use diesel::result::Error::DatabaseError;
use rocket::serde::{Deserialize, Serialize};
use rocket::{
    http::{ContentType, Status},
    response::Responder,
    serde::json::serde_json,
    Response,
};

#[derive(Deserialize, Serialize)]
pub struct ApiError {
    pub code: ApiErrorCodes,
    pub scope: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum ApiErrorCodes {
    InvalidField,
    InternalError,
    Forbidden,
}

impl From<DieselError> for ApiError {
    fn from(error: DieselError) -> ApiError {
        match error {
            DatabaseError(UniqueViolation, _) => ApiError {
                code: ApiErrorCodes::InvalidField,
                scope: None,
                message: None,
            },
            _ => ApiError {
                code: ApiErrorCodes::InternalError,
                scope: None,
                message: None,
            },
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        match self.code {
            ApiErrorCodes::InvalidField => {
                let mut builder = Response::build();
                builder.header(ContentType::JSON).status(Status::BadRequest);
                match serde_json::to_string_pretty(&self) {
                    Ok(json) => {
                        builder.sized_body(json.len(), Cursor::new(json));
                        ()
                    }
                    Err(_) => (),
                };
                builder.ok()
            }
            ApiErrorCodes::InternalError => {
                Response::build().status(Status::InternalServerError).ok()
            }
            ApiErrorCodes::Forbidden => Response::build().status(Status::Forbidden).ok(),
        }
    }
}
