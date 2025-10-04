use std::marker::PhantomData;

use crate::{Actor, ActorContext, ActorPath, ActorRef, ActorSpawner, ActorSystem, Mailbox};

#[derive(Debug)]
pub struct ActorProps<A: Actor, S: ActorSpawner, M: Mailbox<A>> {
    spawner: S,
    mailbox: Option<M>,
    _actor: PhantomData<A>,
}

impl<A: Actor, S: ActorSpawner, M: Mailbox<A>> ActorProps<A, S, M> {
    pub fn new(spawner: S, mailbox: M) -> Self {
        Self {
            spawner,
            mailbox: Some(mailbox),
            _actor: PhantomData,
        }
    }

    pub(crate) fn spawn(&mut self, system: ActorSystem, path: ActorPath, actor: A) -> ActorRef<A> {
        let ctx = ActorContext {
            path: path.clone(),
            system,
            _private: PhantomData,
        };

        let mailbox = self.mailbox.take().unwrap();
        self.spawner.spawn(ctx, actor, mailbox)
    }
}
