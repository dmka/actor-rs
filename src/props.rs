use std::marker::PhantomData;

use crate::{Actor, ActorContext, ActorPath, ActorRef, ActorSpawner, ActorSystem, Mailbox};

#[derive(Debug)]
pub struct ActorProps<A, F, S, M>
where
    A: Actor,
    F: Fn() -> A,
    S: Fn() -> Box<dyn ActorSpawner<A>>,
    M: Fn() -> Box<dyn Mailbox<A>>,
{
    actor_fn: F,
    spawner_fn: S,
    mailbox_fn: M,
}

impl<A, F, S, M> ActorProps<A, F, S, M>
where
    A: Actor,
    F: Fn() -> A,
    S: Fn() -> Box<dyn ActorSpawner<A>>,
    M: Fn() -> Box<dyn Mailbox<A>>,
{
    pub fn new(actor_fn: F, spawner_fn: S, mailbox_fn: M) -> Self {
        Self {
            actor_fn,
            spawner_fn,
            mailbox_fn,
        }
    }

    pub fn new_actor(&self) -> A {
        (self.actor_fn)()
    }

    pub fn new_spawner(&self) -> Box<dyn ActorSpawner<A>> {
        (self.spawner_fn)()
    }

    pub fn new_mailbox(&self) -> Box<dyn Mailbox<A>> {
        (self.mailbox_fn)()
    }

    pub(crate) fn spawn(&mut self, system: ActorSystem, path: ActorPath) -> ActorRef<A> {
        let ctx = ActorContext {
            path: path.clone(),
            system,
            _private: PhantomData,
        };

        let actor = self.new_actor();
        let mailbox = self.new_mailbox();
        let spawner = self.new_spawner();

        spawner.spawn(ctx, actor, mailbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spawner::DefaultActorSpawner;
    use crate::{DefaultMailbox, prelude::*};
    use async_trait::async_trait;

    struct TestActor;
    #[async_trait]
    impl Actor for TestActor {}

    #[tokio::test]
    async fn test1() {
        let system = ActorSystem::new();

        let mut sut = ActorProps::new(
            || TestActor,
            || Box::new(DefaultActorSpawner::<TestActor>::new()),
            || Box::new(DefaultMailbox::<TestActor>::new(10)),
        );

        sut.spawn(system, ActorPath::new("test"));
    }
}
