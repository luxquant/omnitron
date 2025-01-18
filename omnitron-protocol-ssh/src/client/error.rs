use std::error::Error;

use omnitron_common::OmnitronError;

#[derive(thiserror::Error, Debug)]
pub enum SshClientError {
    #[error("mpsc error")]
    MpscError,
    #[error("russh error: {0}")]
    Russh(#[from] russh::Error),
    #[error(transparent)]
    Omnitron(#[from] OmnitronError),
    #[error(transparent)]
    Other(Box<dyn Error + Send + Sync>),
}

impl SshClientError {
    pub fn other<E: Error + Send + Sync + 'static>(err: E) -> Self {
        Self::Other(Box::new(err))
    }
}
