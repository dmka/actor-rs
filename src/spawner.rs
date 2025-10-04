use std::marker::PhantomData;

use crate::{Actor, ActorContext, ActorRef, Mailbox};

pub trait ActorSpawner<A: Actor> {
    fn spawn(&self, ctx: ActorContext, actor: A, mailbox: Box<dyn Mailbox<A>>) -> ActorRef<A>;
}

#[derive(Default)]
pub struct DefaultActorSpawner<A: Actor> {
    _actor: PhantomData<A>,
}

impl<A: Actor> DefaultActorSpawner<A> {
    pub fn new() -> Self {
        Self {
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> ActorSpawner<A> for DefaultActorSpawner<A> {
    fn spawn(
        &self,
        mut ctx: ActorContext,
        mut actor: A,
        mut mailbox: Box<dyn Mailbox<A>>,
    ) -> ActorRef<A> {
        let actor_ref = ActorRef::new(ctx.path.clone(), mailbox.take_sender());

        tokio::spawn(async move {
            if actor.started(&mut ctx).await.is_err() {
                return;
            }

            mailbox.process_messages(&mut ctx, &mut actor).await;

            ctx.system.stop_actor(&ctx.path).await;

            actor.stopped(&mut ctx).await;
        });

        actor_ref
    }
}
