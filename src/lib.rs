mod context;
mod error;
mod handler;
mod mailbox;
pub mod prelude;
mod props;
mod reference;
mod spawner;
mod system;

use async_trait::async_trait;

pub use context::ActorContext;
pub use error::{ActorError, Result};
pub use mailbox::{
    ActorPath, ActorRef, BoxedMessageHandler, DefaultMailbox, Mailbox, MessageHandler,
    MessageHandlerResult, MessageProcessor, Receiver, Sender, WeakSender,
};
pub use props::ActorProps;
pub use spawner::{ActorSpawner, DefaultActorSpawner};
pub use system::ActorSystem;

pub trait Message: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

pub enum StoppingResult {
    Stop,
    Cancel,
}

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    async fn started(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }

    async fn restarting(
        &mut self,
        ctx: &mut ActorContext,
        _error: Option<&ActorError>,
    ) -> Result<()> {
        self.started(ctx).await
    }

    async fn stopping(&mut self, _ctx: &mut ActorContext, _reason: &str) -> StoppingResult {
        StoppingResult::Stop
    }

    async fn stopped(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
pub trait Handler<M: Message>: Actor {
    async fn handle(&mut self, msg: M, ctx: &mut ActorContext) -> M::Response;
}
