use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;

use crate::actor::{
    Actor, ActorContext, ActorError, ActorPath, ActorProps, ActorRef, DefaultActorSpawner,
    DefaultMailbox, Mailbox, spawner::ActorSpawner,
};

#[derive(Clone)]
pub struct ActorSystem {
    name: String,
    actors: Arc<RwLock<HashMap<ActorPath, Box<dyn Any + Send + Sync + 'static>>>>,
}

impl ActorSystem {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let actors = Arc::new(RwLock::new(HashMap::new()));
        ActorSystem { name, actors }
    }

    pub async fn spawn_actor<A: Actor>(
        &self,
        name: &str,
        actor: A,
        buffer: usize,
    ) -> Result<ActorRef<A>, ActorError> {
        let props = ActorProps::new(DefaultActorSpawner::new(), DefaultMailbox::<A>::new(buffer));

        self.spawn_actor_path(ActorPath(name.into()), actor, props)
            .await
    }

    pub(crate) async fn spawn_actor_path<A: Actor, S: ActorSpawner, M: Mailbox<A>>(
        &self,
        path: ActorPath,
        actor: A,
        mut props: ActorProps<A, S, M>,
    ) -> Result<ActorRef<A>, ActorError> {
        let mut actors = self.actors.write().await;
        if actors.contains_key(&path) {
            return Err(ActorError::Exists(path));
        }

        let actor_ref = props.spawn(self.clone(), path, actor);

        let path = actor_ref.path().clone();
        let any = Box::new(actor_ref.clone());

        actors.insert(path, any);

        Ok(actor_ref)
    }

    pub async fn get_actor<A: Actor>(&self, path: &ActorPath) -> Option<ActorRef<A>> {
        let actors = self.actors.read().await;
        actors
            .get(path)
            .and_then(|any| any.downcast_ref::<ActorRef<A>>().cloned())
    }

    pub async fn stop_actor(&self, path: &ActorPath) {
        let mut paths: Vec<ActorPath> = vec![path.clone()];
        {
            let running_actors = self.actors.read().await;
            for running in running_actors.keys() {
                if path.0 != running.0 && running.0.starts_with(&path.0) {
                    paths.push(running.clone());
                }
            }
        }
        paths.sort_unstable();
        paths.reverse();
        let mut actors = self.actors.write().await;
        for path in &paths {
            println!("removing {path}");
            actors.remove(path);
        }
    }
}
