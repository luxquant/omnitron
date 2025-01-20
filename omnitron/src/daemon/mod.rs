pub(crate) mod commands;
pub(crate) mod daemonizer;
pub(crate) mod rpc;
pub(crate) mod server;

use futures::prelude::*;
use futures::StreamExt;
use global_placeholders::global;
use omnitron_rpc::server::{BaseChannel, Channel};
use omnitron_rpc::tokio_serde::formats::Bincode;
use omnitron_rpc::tokio_util::codec::length_delimited::LengthDelimitedCodec;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::oneshot;

use crate::daemon::rpc::DaemonService;
use crate::gate::config::{create_config, load_config};
use crate::logging::init_logging;

async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
  tokio::spawn(fut);
}

pub(crate) async fn daemon_main(cli: &crate::Cli) {
  if let Err(e) = create_config().await {
    panic!("Failed to create gate config: {:?}", e);
  }
  init_logging(load_config(false).ok().as_ref(), cli).await;
  let _ = std::fs::remove_file(global!("omnitron.sock"));

  let listener = UnixListener::bind(global!("omnitron.sock")).unwrap();
  let codec_builder = LengthDelimitedCodec::builder();

  let server_task = tokio::spawn(async move {
    loop {
      let (conn, _addr) = listener.accept().await.unwrap();
      let framed = codec_builder.new_framed(conn);
      let transport = omnitron_rpc::serde_transport::new(framed, Bincode::default());

      let fut = BaseChannel::with_defaults(transport)
        .execute(crate::daemon::server::DaemonServiceServer.serve())
        .for_each(spawn);
      tokio::spawn(fut);
    }
  });

  let (shutdown_tx, shutdown_rx) = oneshot::channel();

  // Handling signals
  let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set SIGTERM handler");
  let mut sighup = signal(SignalKind::hangup()).expect("Failed to set SIGHUP handler");

  tokio::select! {
    _ = shutdown_rx => {
        println!("Shutdown signal received through channel.");
    }
    _ = sigterm.recv() => {
        println!("[daemon:{}] stopped (SIGTERM received) ", std::process::id());
        omnitron_pm::daemon::pid::remove();
        let _ = shutdown_tx.send(());
    }
    _ = sighup.recv() => {
        println!("SIGHUP received. Reloading configuration.");
    }
  }

  // Shutdown server
  server_task.abort();
}
