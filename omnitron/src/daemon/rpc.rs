use global_placeholders::global;
use omnitron_rpc::tokio_serde::formats::Bincode;

#[omnitron_rpc::service]
pub(crate) trait DaemonService {
  /// Returns omnitron version.
  async fn version(short: bool) -> String;
}

pub(crate) async fn create_client() -> Result<DaemonServiceClient, anyhow::Error> {
  let transport = omnitron_rpc::serde_transport::unix::connect(&global!("omnitron.sock"), Bincode::default).await?;
  return Ok(DaemonServiceClient::new(omnitron_rpc::client::Config::default(), transport).spawn());
}
