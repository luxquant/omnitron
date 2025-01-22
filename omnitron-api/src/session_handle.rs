use std::any::type_name;
use std::sync::Arc;

use gate_core::{OmnitronServerHandle, SessionHandle};
use poem::error::GetDataError;
use poem::session::Session;
use poem::web::Data;
use poem::{FromRequest, Request, RequestBody};
use tokio::sync::{mpsc, Mutex};

use crate::session::SessionStore;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionHandleCommand {
  Close,
}

pub struct HttpSessionHandle {
  sender: mpsc::UnboundedSender<SessionHandleCommand>,
}

impl HttpSessionHandle {
  pub fn new() -> (Self, mpsc::UnboundedReceiver<SessionHandleCommand>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (HttpSessionHandle { sender }, receiver)
  }
}

impl SessionHandle for HttpSessionHandle {
  fn close(&mut self) {
    let _ = self.sender.send(SessionHandleCommand::Close);
  }
}

#[derive(Clone)]
pub struct OmnitronServerHandleFromRequest(pub Arc<Mutex<OmnitronServerHandle>>);

impl std::ops::Deref for OmnitronServerHandleFromRequest {
  type Target = Arc<Mutex<OmnitronServerHandle>>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'a> FromRequest<'a> for OmnitronServerHandleFromRequest {
  async fn from_request(req: &'a Request, _: &mut RequestBody) -> poem::Result<Self> {
    let sm = Data::<&Arc<Mutex<SessionStore>>>::from_request_without_body(req).await?;
    let session = <&Session>::from_request_without_body(req).await?;
    Ok(
      sm.lock()
        .await
        .handle_for(session)
        .map(OmnitronServerHandleFromRequest)
        .ok_or_else(|| GetDataError(type_name::<OmnitronServerHandle>()))?,
    )
  }
}

impl From<Arc<Mutex<OmnitronServerHandle>>> for OmnitronServerHandleFromRequest {
  fn from(handle: Arc<Mutex<OmnitronServerHandle>>) -> Self {
    OmnitronServerHandleFromRequest(handle)
  }
}
