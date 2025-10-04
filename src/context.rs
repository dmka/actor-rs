use std::marker::PhantomData;

use crate::{
    Actor, ActorPath, ActorProps, ActorRef, ActorSpawner, ActorSystem, DefaultActorSpawner,
    DefaultMailbox, Mailbox, Result,
};

#[derive(Debug)]
pub struct ActorContext {
    pub path: ActorPath,
    pub system: ActorSystem,
    pub(crate) _private: PhantomData<()>,
}

impl ActorContext {
    pub async fn spawn<A: Actor, F: Fn() -> A>(
        &self,
        name: &str,
        actor_fn: F,
        buffer: usize,
    ) -> Result<ActorRef<A>> {
        let props = ActorProps::new(
            actor_fn,
            || Box::new(DefaultActorSpawner::new()),
            || Box::new(DefaultMailbox::<A>::new(buffer)),
        );

        let child = self.path.join(name);
        self.system.spawn_path(child, props).await
    }

    pub async fn spawn_props<A, F, S, M>(
        &self,
        name: &str,
        props: ActorProps<A, F, S, M>,
    ) -> Result<ActorRef<A>>
    where
        A: Actor,
        F: Fn() -> A,
        S: Fn() -> Box<dyn ActorSpawner<A>>,
        M: Fn() -> Box<dyn Mailbox<A>>,
    {
        let child = self.path.join(name);
        self.system.spawn_path(child, props).await
    }

    pub async fn get<A: Actor>(&self, name: &str) -> Option<ActorRef<A>> {
        let child = self.path.join(name);
        self.system.get(&child).await
    }

    pub async fn stop(&self, name: &str) {
        let child = self.path.join(name);
        self.system.stop_actor(&child).await;
    }
}
