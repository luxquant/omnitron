use std::sync::Arc;

use omnitron_db_entities::KnownHost;
use omnitron_gate_common::OmnitronError;
use poem::web::Data;
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, OpenApi};
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::sync::Mutex;

use super::AnySecurityScheme;

pub struct Api;

#[derive(ApiResponse)]
enum GetSSHKnownHostsResponse {
  #[oai(status = 200)]
  Ok(Json<Vec<KnownHost::Model>>),
}

#[OpenApi]
impl Api {
  #[oai(path = "/ssh/known-hosts", method = "get", operation_id = "get_ssh_known_hosts")]
  async fn api_ssh_get_all_known_hosts(
    &self,
    db: Data<&Arc<Mutex<DatabaseConnection>>>,
    _auth: AnySecurityScheme,
  ) -> Result<GetSSHKnownHostsResponse, OmnitronError> {
    use omnitron_db_entities::KnownHost;

    let db = db.lock().await;
    let hosts = KnownHost::Entity::find().all(&*db).await?;
    Ok(GetSSHKnownHostsResponse::Ok(Json(hosts)))
  }
}
