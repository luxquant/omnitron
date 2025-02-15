use std::sync::Arc;

use omnitron_gate_common::OmnitronError;
use poem::web::Data;
use poem_openapi::param::Path;
use poem_openapi::{ApiResponse, OpenApi};
use sea_orm::{DatabaseConnection, EntityTrait, ModelTrait};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::AnySecurityScheme;
pub struct Api;

#[derive(ApiResponse)]
enum DeleteSSHKnownHostResponse {
  #[oai(status = 204)]
  Deleted,

  #[oai(status = 404)]
  NotFound,
}

#[OpenApi]
impl Api {
  #[oai(path = "/ssh/known-hosts/:id", method = "delete", operation_id = "delete_ssh_known_host")]
  async fn api_ssh_delete_known_host(
    &self,
    db: Data<&Arc<Mutex<DatabaseConnection>>>,
    id: Path<Uuid>,
    _auth: AnySecurityScheme,
  ) -> Result<DeleteSSHKnownHostResponse, OmnitronError> {
    use omnitron_db_entities::KnownHost;
    let db = db.lock().await;

    let known_host = KnownHost::Entity::find_by_id(id.0).one(&*db).await?;

    match known_host {
      Some(known_host) => {
        known_host.delete(&*db).await?;
        Ok(DeleteSSHKnownHostResponse::Deleted)
      }
      None => Ok(DeleteSSHKnownHostResponse::NotFound),
    }
  }
}
