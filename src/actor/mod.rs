#![allow(dead_code)]

mod mailbox;
mod spawner;
mod system;

use std::{fmt::Display, marker::PhantomData};

use anyhow::Result;
use async_trait::async_trait;
use thiserror::Error;

pub use mailbox::ActorRef;
pub use spawner::{ActorSpawner, DefaultActorSpawner};
pub use system::ActorSystem;

pub trait Message: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    async fn pre_start(&mut self, _ctx: &mut ActorContext) -> Result<(), ActorError> {
        Ok(())
    }

    async fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        _error: Option<&ActorError>,
    ) -> Result<(), ActorError> {
        self.pre_start(ctx).await
    }

    async fn post_stop(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
pub trait Handler<M: Message>: Actor {
    async fn handle(&mut self, msg: M, ctx: &mut ActorContext) -> M::Response;
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct ActorPath(pub String);

impl Display for ActorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct ActorContext {
    pub path: ActorPath,
    pub system: system::ActorSystem,
    _private: PhantomData<()>,
}

impl ActorContext {
    pub async fn create_child<A: Actor, S: ActorSpawner<A>>(
        &self,
        name: &str,
        actor: A,
        spawner: S,
    ) -> Result<ActorRef<A>, ActorError> {
        let path = ActorPath(format!("{}/{}", self.path, name));
        self.system.spawn_actor_path(path, actor, spawner).await
    }

    pub async fn get_child<A: Actor>(&self, name: &str) -> Option<ActorRef<A>> {
        let path = ActorPath(format!("{}/{}", self.path, name));
        self.system.get_actor(&path).await
    }

    pub async fn stop_child(&self, name: &str) {
        let path = ActorPath(format!("{}/{}", self.path, name));
        self.system.stop_actor(&path).await;
    }
}

#[async_trait]
trait MessageProcessor {
    async fn process_message();
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
