use thiserror::Error;

use crate::ActorPath;

pub type Result<T> = std::result::Result<T, ActorError>;

#[derive(Debug, Error)]
pub enum ActorError {
    #[error("Actor exists")]
    Exists(ActorPath),

    #[error("Actor creation failed")]
    CreateError(String),

    #[error("Sending message failed")]
    SendError(String),

    #[error("Actor runtime error")]
    RuntimeError(anyhow::Error),
}
