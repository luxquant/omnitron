use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;

use super::structs::ErrorMessage;

pub(crate) type NotFound = status::NotFound<Json<ErrorMessage>>;
pub(crate) type GenericError = status::Custom<Json<ErrorMessage>>;

pub(crate) fn create_status(code: Status) -> Json<ErrorMessage> {
  Json(ErrorMessage {
    code,
    message: code.to_string(),
  })
}

pub(crate) fn generic_error(code: Status, message: String) -> GenericError {
  status::Custom(code, Json(ErrorMessage { code, message }))
}

pub(crate) fn not_found(msg: &str) -> NotFound {
  status::NotFound(Json(ErrorMessage {
    code: Status::NotFound,
    message: msg.to_string(),
  }))
}
