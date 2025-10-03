use std::marker::PhantomData;

use crate::actor::{
    Actor, ActorContext, ActorRef, mailbox::DefaultMailbox, mailbox::MessageProcessor,
};

pub trait ActorSpawner<A: Actor>: Clone {
    fn spawn(&self, ctx: ActorContext, actor: A) -> ActorRef<A>;
}

pub struct DefaultActorSpawner<A: Actor> {
    pub(crate) _actor: PhantomData<A>,
}

impl<A: Actor> DefaultActorSpawner<A> {
    pub fn new() -> Self {
        Self {
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> Clone for DefaultActorSpawner<A> {
    fn clone(&self) -> Self {
        Self {
            _actor: self._actor,
        }
    }
}

impl<A: Actor> ActorSpawner<A> for DefaultActorSpawner<A> {
    fn spawn(&self, mut ctx: ActorContext, mut actor: A) -> ActorRef<A> {
        let mut mailbox = DefaultMailbox::new();
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
