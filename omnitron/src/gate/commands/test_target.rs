use anyhow::Result;
use omnitron_gate_common::TargetOptions;
use omnitron_gate_core::{ProtocolServer, Services, TargetTestError};
use tracing::*;

use crate::gate::config::load_config;

pub(crate) async fn command(target_name: &String) -> Result<()> {
  let config = load_config(true)?;
  let services = Services::new(config.clone(), None).await?;

  let Some(target) = services
    .config_provider
    .lock()
    .await
    .list_targets()
    .await?
    .iter()
    .find(|x| &x.name == target_name)
    .cloned()
  else {
    error!("Target not found: {}", target_name);
    return Ok(());
  };

  let s: Box<dyn ProtocolServer> = match target.options {
    TargetOptions::Ssh(_) => Box::new(omnitron_gate_protocol_ssh::SSHProtocolServer::new(&services).await?),
    TargetOptions::Http(_) => Box::new(omnitron_gate_protocol_http::HTTPProtocolServer::new(&services).await?),
    TargetOptions::MySql(_) => Box::new(omnitron_gate_protocol_mysql::MySQLProtocolServer::new(&services).await?),
    TargetOptions::Postgres(_) => Box::new(omnitron_gate_protocol_postgres::PostgresProtocolServer::new(&services).await?),
    TargetOptions::WebAdmin(_) => {
      error!("Unsupported target type");
      return Ok(());
    }
  };

  match s.test_target(target).await {
    Err(TargetTestError::AuthenticationError) => {
      error!("Authentication failed");
    }
    Err(TargetTestError::ConnectionError(error)) => {
      error!(?error, "Connection error");
    }
    Err(TargetTestError::Io(error)) => {
      error!(?error, "I/O error");
    }
    Err(TargetTestError::Misconfigured(error)) => {
      error!(?error, "Misconfigured");
    }
    Err(TargetTestError::Unreachable) => {
      error!("Target is unreachable");
    }
    Ok(()) => {
      info!("Connection successful!");
      return Ok(());
    }
  }

  anyhow::bail!("Connection test failed")
}
