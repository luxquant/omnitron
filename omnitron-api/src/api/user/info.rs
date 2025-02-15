use omnitron_db_entities::Parameters;
use omnitron_gate_common::OmnitronError;
use omnitron_gate_core::Services;
use poem::session::Session;
use poem::web::Data;
use poem::Request;
use poem_openapi::payload::Json;
use poem_openapi::{ApiResponse, Object, OpenApi};
use serde::Serialize;

use crate::common::{SessionAuthorization, SessionExt};

pub struct Api;

#[derive(Serialize, Object)]
pub struct PortsInfo {
  ssh: Option<u16>,
  http: Option<u16>,
  mysql: Option<u16>,
  postgres: Option<u16>,
}

#[derive(Serialize, Object)]
pub struct Info {
  version: String,
  username: Option<String>,
  selected_target: Option<String>,
  external_host: Option<String>,
  ports: PortsInfo,
  authorized_via_ticket: bool,
  own_credential_management_allowed: bool,
}

#[derive(ApiResponse)]
enum InstanceInfoResponse {
  #[oai(status = 200)]
  Ok(Json<Info>),
}

#[OpenApi]
impl Api {
  #[oai(path = "/info", method = "get", operation_id = "get_info")]
  async fn api_get_info(
    &self,
    req: &Request,
    session: &Session,
    services: Data<&Services>,
  ) -> Result<InstanceInfoResponse, OmnitronError> {
    let config = services.config.lock().await;
    let external_host = config
      .construct_external_url(Some(req), None)
      .ok()
      .as_ref()
      .and_then(|x| x.host())
      .map(|x| x.to_string());

    let parameters = Parameters::Entity::get(&*services.db.lock().await).await?;

    Ok(InstanceInfoResponse::Ok(Json(Info {
      version: env!("CARGO_PKG_VERSION").to_string(),
      username: session.get_username(),
      selected_target: session.get_target_name(),
      external_host,
      authorized_via_ticket: matches!(session.get_auth(), Some(SessionAuthorization::Ticket { .. })),
      ports: if session.is_authenticated() {
        PortsInfo {
          ssh: Some(config.store.ssh.external_port()),
          http: Some(config.store.http.external_port()),
          mysql: Some(config.store.mysql.external_port()),
          postgres: Some(config.store.postgres.external_port()),
        }
      } else {
        PortsInfo {
          ssh: None,
          http: None,
          mysql: None,
          postgres: None,
        }
      },
      own_credential_management_allowed: parameters.allow_own_credential_management,
    })))
  }
}
