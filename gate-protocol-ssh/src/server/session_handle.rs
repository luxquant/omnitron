use gate_core::SessionHandle;
use tokio::sync::mpsc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionHandleCommand {
  Close,
}

pub struct SSHSessionHandle {
  sender: mpsc::UnboundedSender<SessionHandleCommand>,
}

impl SSHSessionHandle {
  pub fn new() -> (Self, mpsc::UnboundedReceiver<SessionHandleCommand>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (SSHSessionHandle { sender }, receiver)
  }
}

impl SessionHandle for SSHSessionHandle {
  fn close(&mut self) {
    let _ = self.sender.send(SessionHandleCommand::Close);
  }
}
