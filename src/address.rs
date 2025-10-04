use std::fmt::Display;

use crate::{
    Actor, Handler, MailboxSender, Message, Result,
    handler::{PoisonMessage, SystemHandler},
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ActorPath(pub String);

impl Display for ActorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct ActorRef<A: Actor> {
    path: ActorPath,
    sender: MailboxSender<A>,
}

impl<A: Actor> ActorRef<A> {
    pub fn new(path: ActorPath, sender: MailboxSender<A>) -> Self {
        ActorRef { path, sender }
    }

    pub fn path(&self) -> &ActorPath {
        &self.path
    }

    pub async fn tell<M>(&self, msg: M) -> Result<()>
    where
        M: Message,
        A: Handler<M>,
    {
        self.sender.tell(msg).await
    }

    pub async fn ask<M>(&self, msg: M) -> Result<M::Response>
    where
        M: Message,
        A: Handler<M>,
    {
        self.sender.ask(msg).await
    }

    pub async fn poison(&self) -> Result<()> {
        self.sys_tell(PoisonMessage).await
    }

    async fn sys_tell<M>(&self, msg: M) -> Result<()>
    where
        M: Message,
        A: SystemHandler<M>,
    {
        self.sender.sys_tell(msg).await
    }

    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            sender: self.sender.clone(),
        }
    }
}
