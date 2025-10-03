use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};
use tokio::sync::RwLock;

use crate::actor::{Actor, ActorContext, ActorError, ActorPath, ActorRef, ActorSpawner};

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

    pub(crate) async fn spawn_actor_path<A: Actor, S: ActorSpawner<A>>(
        &self,
        path: ActorPath,
        actor: A,
        spawner: S,
    ) -> Result<ActorRef<A>, ActorError> {
        println!("Creating actor '{:?}' on system '{}'...", &path, &self.name);

        let mut actors = self.actors.write().await;
        if actors.contains_key(&path) {
            return Err(ActorError::Exists(path));
        }

        let ctx = ActorContext {
            path: path.clone(),
            system: self.clone(),
            _private: PhantomData,
        };

        let actor_ref = spawner.spawn(ctx, actor);

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
        println!("Stopping actor '{}' on system '{}'...", &path, &self.name);
        let mut paths: Vec<ActorPath> = vec![path.clone()];
        {
            let running_actors = self.actors.read().await;
            for running in running_actors.keys() {
                if running.0.starts_with(&path.0) {
                    paths.push(running.clone());
                }
            }
        }
        paths.sort_unstable();
        paths.reverse();
        let mut actors = self.actors.write().await;
        for path in &paths {
            actors.remove(path);
        }
    }
}
