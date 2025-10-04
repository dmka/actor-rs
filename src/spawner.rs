use crate::{Actor, ActorContext, ActorRef, Mailbox};

pub trait ActorSpawner {
    fn spawn<A: Actor, M: Mailbox<A>>(
        &self,
        ctx: ActorContext,
        actor: A,
        mailbox: M,
    ) -> ActorRef<A>;
}

#[derive(Default)]
pub struct DefaultActorSpawner;

impl DefaultActorSpawner {
    pub fn new() -> Self {
        Self
    }
}

impl ActorSpawner for DefaultActorSpawner {
    fn spawn<A: Actor, M: Mailbox<A>>(
        &self,
        mut ctx: ActorContext,
        mut actor: A,
        mut mailbox: M,
    ) -> ActorRef<A> {
        let actor_ref = ActorRef::new(ctx.path.clone(), mailbox.take_sender());

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
