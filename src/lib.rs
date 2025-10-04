mod address;
mod context;
mod error;
mod handler;
mod mailbox;
pub mod prelude;
mod props;
mod spawner;
mod system;

use async_trait::async_trait;

pub use context::ActorContext;
pub use error::{ActorError, Result};
pub use mailbox::{
    ActorPath, ActorRef, BoxedMessageHandler, DefaultMailbox, Mailbox, MailboxReceiver,
    MailboxSender, MessageHandler, MessageHandlerResult, MessageProcessor,
};
pub use props::ActorProps;
pub use spawner::{ActorSpawner, DefaultActorSpawner};
pub use system::ActorSystem;

pub trait Message: Send + Sync + 'static {
    type Response: Send + Sync + 'static;
}

#[async_trait]
pub trait Actor: Send + Sync + 'static {
    async fn pre_start(&mut self, _ctx: &mut ActorContext) -> Result<()> {
        Ok(())
    }

    async fn pre_restart(
        &mut self,
        ctx: &mut ActorContext,
        _error: Option<&ActorError>,
    ) -> Result<()> {
        self.pre_start(ctx).await
    }

    async fn post_stop(&mut self, _ctx: &mut ActorContext) {}
}

#[async_trait]
pub trait Handler<M: Message>: Actor {
    async fn handle(&mut self, msg: M, ctx: &mut ActorContext) -> M::Response;
}
