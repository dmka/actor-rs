use std::marker::PhantomData;

use crate::actor::{Actor, ActorContext, ActorRef, DefaultMailbox, Mailbox, MessageProcessor};

pub trait ActorSpawner {
    fn spawn<A: Actor, M: Mailbox<A>>(
        &self,
        ctx: ActorContext,
        actor: A,
        mailbox: M,
    ) -> ActorRef<A>;
}

pub struct DefaultActorSpawner {
    buffer: usize,
}

impl DefaultActorSpawner {
    pub fn new(buffer: usize) -> Self {
        Self { buffer }
    }
}

impl ActorSpawner for DefaultActorSpawner {
    fn spawn<A: Actor, M: Mailbox<A>>(
        &self,
        mut ctx: ActorContext,
        mut actor: A,
        mut mailbox: M,
    ) -> ActorRef<A> {
        let actor_ref = ActorRef::new(ctx.path.clone(), mailbox.sender());

        tokio::spawn(async move {
            if actor.pre_start(&mut ctx).await.is_err() {
                return;
            }

            mailbox.process_messages(&mut ctx, &mut actor).await;

            actor.post_stop(&mut ctx).await;
        });

        actor_ref
    }
}
