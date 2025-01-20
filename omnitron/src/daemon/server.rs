use omnitron_rpc::context;

use super::rpc::DaemonService;

#[derive(Clone)]
pub(crate) struct DaemonServiceServer;

impl DaemonService for DaemonServiceServer {
  async fn version(self, _: context::Context, short: bool) -> String {
    crate::helpers::get_version(short)
  }
}
