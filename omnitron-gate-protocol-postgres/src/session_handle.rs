use omnitron_gate_core::SessionHandle;
use tokio::sync::mpsc;

pub struct PostgresSessionHandle {
  abort_tx: mpsc::UnboundedSender<()>,
}

impl PostgresSessionHandle {
  pub fn new() -> (Self, mpsc::UnboundedReceiver<()>) {
    let (abort_tx, abort_rx) = mpsc::unbounded_channel();
    (PostgresSessionHandle { abort_tx }, abort_rx)
  }
}

impl SessionHandle for PostgresSessionHandle {
  fn close(&mut self) {
    let _ = self.abort_tx.send(());
  }
}
