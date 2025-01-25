// Declaration of modules that will be used in this file
pub(crate) mod commands;
pub(crate) mod daemonizer;
pub(crate) mod rpc;
pub(crate) mod server;

// Import necessary libraries and modules
use anyhow::Result;
use futures::prelude::*;
use futures::StreamExt;
use global_placeholders::global;
use omnitron_gate_core::db::cleanup_db;
use omnitron_gate_core::logging::install_database_logger;
use omnitron_gate_core::{ProtocolServer, Services};
use omnitron_gate_protocol_http::HTTPProtocolServer;
use omnitron_rpc::server::{BaseChannel, Channel};
use omnitron_rpc::tokio_serde::formats::Bincode;
use omnitron_rpc::tokio_util::codec::length_delimited::LengthDelimitedCodec;
#[cfg(target_os = "linux")]
use sd_notify::NotifyState;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
// use tokio::sync::oneshot;
use tracing::*;

// Import local modules
use crate::daemon::rpc::DaemonService;
use crate::gate::config::{create_config, load_config, watch_config};
use crate::logging::init_logging;

// Asynchronous function to spawn tasks
async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
  tokio::spawn(fut); // Spawn the task in a separate thread
}

// Main function of the daemon
pub(crate) async fn daemon_main(cli: &crate::Cli, enable_admin_token: bool) -> Result<()> {
  // Attempt to create configuration, panic if it fails
  if let Err(e) = create_config().await {
    panic!("Failed to create gate config: {:?}", e);
  }

  // Initialize logging with the loaded configuration
  init_logging(load_config(false).ok().as_ref(), cli).await;

  // Remove the socket file if it exists
  let _ = std::fs::remove_file(global!("omnitron.sock"));

  // Create a UnixListener to listen for connections
  let listener = UnixListener::bind(global!("omnitron.sock")).unwrap();
  // Create a builder for the codec
  let codec_builder = LengthDelimitedCodec::builder();

  // Server task that will handle incoming connections
  let server_task = tokio::spawn(async move {
    loop {
      // Wait for an incoming connection
      let (conn, _addr) = listener.accept().await.unwrap();
      // Wrap the connection in frames
      let framed = codec_builder.new_framed(conn);
      // Create a transport using Bincode
      let transport = omnitron_rpc::serde_transport::new(framed, Bincode::default());

      // Create and execute a channel to handle RPC
      let fut = BaseChannel::with_defaults(transport)
        .execute(crate::daemon::server::DaemonServiceServer.serve())
        .for_each(spawn);
      // Spawn the task in a separate thread
      tokio::spawn(fut);
    }
  });

  // ---

  let admin_token = enable_admin_token.then(|| {
    std::env::var("OMNITRON_ADMIN_TOKEN").unwrap_or_else(|_| {
      error!("`OMNITRON_ADMIN_TOKEN` env variable must set when using --enable-admin-token");
      std::process::exit(1);
    })
  });

  let config = match load_config(true) {
    Ok(config) => config,
    Err(error) => {
      error!(?error, "Failed to load config file");
      std::process::exit(1);
    }
  };

  let services = Services::new(config.clone(), admin_token).await?;

  install_database_logger(services.db.clone());

  let mut protocol_futures = futures::stream::FuturesUnordered::new();

  protocol_futures.push(
    HTTPProtocolServer::new(&services)
      .await?
      .run(config.store.http.listen.clone()),
  );

  tokio::spawn({
    let services = services.clone();
    async move {
      loop {
        let retention = { services.config.lock().await.store.log.retention };
        let interval = retention / 10;
        #[allow(clippy::explicit_auto_deref)]
        match cleanup_db(&mut *services.db.lock().await, &retention).await {
          Err(error) => error!(?error, "Failed to cleanup the database"),
          Ok(_) => debug!("Database cleaned up, next in {:?}", interval),
        }
        tokio::time::sleep(interval).await;
      }
    }
  });

  #[cfg(target_os = "linux")]
  if let Ok(true) = sd_notify::booted() {
    use std::time::Duration;
    tokio::spawn(async {
      if let Err(error) = async {
        sd_notify::notify(false, &[NotifyState::Ready])?;
        loop {
          sd_notify::notify(false, &[NotifyState::Watchdog])?;
          tokio::time::sleep(Duration::from_secs(15)).await;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
      }
      .await
      {
        error!(?error, "Failed to communicate with systemd");
      }
    });
  }

  drop(config);

  tokio::spawn(watch_config_and_reload(services.clone()));

  // // Create a channel to manage shutdown
  // let (shutdown_tx, shutdown_rx) = oneshot::channel();

  // Signal handling
  let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set SIGTERM handler");
  let mut sighup = signal(SignalKind::hangup()).expect("Failed to set SIGHUP handler");

  loop {
    tokio::select! {
      // _ = shutdown_rx => {
      //   println!("Shutdown signal received through channel."); // Message for shutdown through channel
      // }
      _ = sigterm.recv() => {
          println!("[daemon:{}] stopped (SIGTERM received) ", std::process::id()); // Message for receiving SIGTERM
          omnitron_pm::daemon::pid::remove(); // Remove PID file
          // let _ = shutdown_tx.send(()); // Send shutdown signal
          break;
      }
      _ = sighup.recv() => {
          println!("SIGHUP received. Reloading configuration."); // Message for receiving SIGHUP
      }
      result = protocol_futures.next() => {
        match result {
          Some(Err(error)) => {
            error!(?error, "Server error");
            // std::process::exit(1);
          },
          None => break,
          _ => (),
        }
      }
    }
  }

  // Abort the server task
  server_task.abort();

  Ok(())
}

pub async fn watch_config_and_reload(services: Services) -> Result<()> {
  let mut reload_event = watch_config(services.config.clone())?;

  while let Ok(()) = reload_event.recv().await {
    let state = services.state.lock().await;
    let mut cp = services.config_provider.lock().await;
    for (id, session) in state.sessions.iter() {
      let mut session = session.lock().await;
      if let (Some(username), Some(target)) = (session.username.as_ref(), session.target.as_ref()) {
        if !cp.authorize_target(username, &target.name).await? {
          warn!(sesson_id=%id, %username, target=&target.name, "Session no longer authorized after config reload");
          session.handle.close();
        }
      }
    }
  }

  Ok(())
}
