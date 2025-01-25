use std::sync::Arc;

use omnitron_gate_common::OmnitronError;
use omnitron_gate_core::{SessionSnapshot, State};
use omnitron_gate_db_entities::Session;
use poem::web::Data;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, OpenApi};
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::AnySecurityScheme;

pub struct Api;

#[allow(clippy::large_enum_variant)]
#[derive(ApiResponse)]
enum GetSessionResponse {
  #[oai(status = 200)]
  Ok(Json<SessionSnapshot>),
  #[oai(status = 404)]
  NotFound,
}

#[derive(ApiResponse)]
enum CloseSessionResponse {
  #[oai(status = 201)]
  Ok,
  #[oai(status = 404)]
  NotFound,
}

#[OpenApi]
impl Api {
  #[oai(path = "/sessions/:id", method = "get", operation_id = "get_session")]
  async fn api_get_session(
    &self,
    db: Data<&Arc<Mutex<DatabaseConnection>>>,
    id: Path<Uuid>,
    _auth: AnySecurityScheme,
  ) -> Result<GetSessionResponse, OmnitronError> {
    let db = db.lock().await;

    let session = Session::Entity::find_by_id(id.0).one(&*db).await?;

    match session {
      Some(session) => Ok(GetSessionResponse::Ok(Json(session.into()))),
      None => Ok(GetSessionResponse::NotFound),
    }
  }

  #[oai(path = "/sessions/:id/close", method = "post", operation_id = "close_session")]
  async fn api_close_session(
    &self,
    state: Data<&Arc<Mutex<State>>>,
    id: Path<Uuid>,
    _auth: AnySecurityScheme,
  ) -> Result<CloseSessionResponse, OmnitronError> {
    let state = state.lock().await;

    if let Some(s) = state.sessions.get(&id) {
      let mut session = s.lock().await;
      session.handle.close();
      Ok(CloseSessionResponse::Ok)
    } else {
      Ok(CloseSessionResponse::NotFound)
    }
  }
}
