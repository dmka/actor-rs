use std::marker::PhantomData;

use crate::actor::{Actor, ActorError, ActorPath, ActorRef, ActorSystem, spawner::ActorSpawner};

pub struct ActorContext {
    pub path: ActorPath,
    pub system: ActorSystem,
    pub(crate) _private: PhantomData<()>,
}

impl ActorContext {
    pub async fn create_child<A: Actor, S: ActorSpawner<A>>(
        &self,
        name: &str,
        actor: A,
        spawner: S,
    ) -> Result<ActorRef<A>, ActorError> {
        let path = ActorPath(format!("{}/{}", self.path, name));
        self.system.spawn_actor_path(path, actor, spawner).await
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
