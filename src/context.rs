use std::marker::PhantomData;

use crate::{Actor, ActorPath, ActorProps, ActorRef, ActorSpawner, ActorSystem, Mailbox, Result};

#[derive(Debug)]
pub struct ActorContext {
    pub path: ActorPath,
    pub system: ActorSystem,
    pub(crate) _private: PhantomData<()>,
}

impl ActorContext {
    pub async fn create_child<A: Actor, S: ActorSpawner, M: Mailbox<A>>(
        &self,
        name: &str,
        actor: A,
        props: ActorProps<A, S, M>,
    ) -> Result<ActorRef<A>> {
        let path = ActorPath(format!("{}/{}", self.path, name));
        self.system.spawn_actor_path(path, actor, props).await
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
