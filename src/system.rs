use async_trait::async_trait;
use std::{any::Any, collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    Actor, ActorContext, ActorError, ActorPath, ActorProps, ActorRef, DefaultActorSpawner,
    DefaultMailbox, Mailbox, Message, MessageHandlerResult, Result, handler::SystemHandler,
    spawner::ActorSpawner,
};

#[derive(Clone, Debug)]
pub struct ActorSystem {
    actors: Arc<RwLock<HashMap<ActorPath, Box<dyn Any + Send + Sync>>>>,
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorSystem {
    pub fn new() -> Self {
        let actors = Arc::new(RwLock::new(HashMap::new()));
        ActorSystem { actors }
    }

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

        self.spawn_props(name, props).await
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
        let path = ActorPath::new(name);
        if path.has_parent() {
            return Err(ActorError::CreateError("invalid actor name".into()));
        }

        self.spawn_path(path, props).await
    }

    pub(crate) async fn spawn_path<A, F, S, M>(
        &self,
        path: ActorPath,
        mut props: ActorProps<A, F, S, M>,
    ) -> Result<ActorRef<A>>
    where
        A: Actor,
        F: Fn() -> A,
        S: Fn() -> Box<dyn ActorSpawner<A>>,
        M: Fn() -> Box<dyn Mailbox<A>>,
    {
        let mut actors = self.actors.write().await;
        if actors.contains_key(&path) {
            return Err(ActorError::Exists(path));
        }

        let actor_ref = props.spawn(self.clone(), path);

        let path = actor_ref.path().clone();
        let any = Box::new(actor_ref.clone());

        actors.insert(path, any);

        Ok(actor_ref)
    }

    pub async fn get<A: Actor>(&self, path: &ActorPath) -> Option<ActorRef<A>> {
        let actors = self.actors.read().await;
        actors
            .get(path)
            .and_then(|any| any.downcast_ref::<ActorRef<A>>().cloned())
    }

    pub async fn stop_actor(&self, path: &ActorPath) {
        let mut base_path = path.to_string();
        base_path.push('/');

        let mut paths = Vec::new();
        paths.push(path.clone());

        let running_actors = self.actors.read().await;
        for running in running_actors.keys() {
            if running.starts_with(&base_path) {
                paths.push(running.clone());
            }
        }
        drop(running_actors);

        paths.sort_unstable();
        paths.reverse();
        let mut actors = self.actors.write().await;
        for path in &paths {
            println!("removing {path}");
            actors.remove(path);
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum SystemMessage {
    Poison,
    Watch { watcher: ActorPath },
    Unwatch { watcher: ActorPath },
    Terminated,
}

pub(crate) struct SystemMessageResponse;

impl Message for SystemMessage {
    type Response = SystemMessageResponse;
}

#[async_trait]
impl<A: Actor> SystemHandler<SystemMessage> for A {
    async fn handle(
        &mut self,
        msg: SystemMessage,
        _ctx: &mut ActorContext,
    ) -> (SystemMessageResponse, MessageHandlerResult) {
        match msg {
            SystemMessage::Poison => (
                SystemMessageResponse,
                MessageHandlerResult::Stop {
                    reason: "poisoned".into(),
                },
            ),
            SystemMessage::Watch { watcher: _ } => todo!(),
            SystemMessage::Terminated => todo!(),
            SystemMessage::Unwatch { watcher: _ } => todo!(),
        }
    }
}
