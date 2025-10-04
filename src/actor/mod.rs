#![allow(dead_code, unused_imports)]

mod context;
mod mailbox;
mod spawner;
mod system;

use std::{fmt::Display, marker::PhantomData};

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;

pub use context::ActorContext;
pub use mailbox::{
    ActorRef, BoxedMessageHandler, DefaultMailbox, Mailbox, MailboxReceiver, MailboxSender,
    MessageHandler, MessageProcessor,
};
pub use spawner::{ActorSpawner, DefaultActorSpawner};
pub use system::ActorSystem;

pub trait Message: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    async fn pre_start(&mut self, _ctx: &mut context::ActorContext) -> Result<(), ActorError> {
        Ok(())
    }

    async fn pre_restart(
        &mut self,
        ctx: &mut context::ActorContext,
        _error: Option<&ActorError>,
    ) -> Result<(), ActorError> {
        self.pre_start(ctx).await
    }

    async fn post_stop(&mut self, _ctx: &mut context::ActorContext) {}
}

#[async_trait]
pub trait Handler<M: Message>: Actor {
    async fn handle(&mut self, msg: M, ctx: &mut context::ActorContext) -> M::Response;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct ActorPath(pub String);

impl Display for ActorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

pub struct ActorProps<A: Actor, S: ActorSpawner, M: Mailbox<A>> {
    spawner: S,
    mailbox: Option<M>,
    actor: PhantomData<A>,
}

impl<A: Actor, S: ActorSpawner, M: Mailbox<A>> ActorProps<A, S, M> {
    pub fn new(spawner: S, mailbox: M) -> Self {
        Self {
            spawner,
            mailbox: Some(mailbox),
            actor: PhantomData,
        }
    }
}
