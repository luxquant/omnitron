use bytes::BytesMut;
use chrono::Local;
use omnitron_gate_common::OmnitronConfig;
use tokio::net::UnixDatagram;
use tracing::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::layer::ValuesLogLayer;

static SKIP_KEY: &str = "is_socket_logging_error";

pub async fn make_socket_logger_layer<S>(config: &OmnitronConfig) -> impl Layer<S>
where
  S: Subscriber + for<'a> LookupSpan<'a>,
{
  let mut socket = None;
  let socket_address = config.store.log.send_to.clone();
  if socket_address.is_some() {
    socket = UnixDatagram::unbound()
      .map_err(|error| {
        println!("Failed to create the log forwarding UDP socket: {error}");
      })
      .ok();
  }

  let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

  let got_socket = socket.is_some();

  let layer = ValuesLogLayer::new(move |mut values| {
    if !got_socket || values.contains_key(&SKIP_KEY) {
      return;
    }
    values.insert("timestamp", Local::now().to_rfc3339());
    let _ = tx.try_send(values);
  });

  if !got_socket {
    return layer;
  }

  tokio::spawn(async move {
    while let Some(values) = rx.recv().await {
      let Some(ref socket) = socket else { return };
      let Some(ref socket_address) = socket_address else {
        return;
      };

      let Ok(serialized) = serde_json::to_vec(&values) else {
        eprintln!("Failed to serialize log entry {values:?}");
        continue;
      };

      let buffer = BytesMut::from(&serialized[..]);
      if let Err(error) = socket.send_to(buffer.as_ref(), socket_address).await {
        error!(%error, is_socket_logging_error=true, "Failed to forward log entry");
      }
    }
  });

  layer
}
