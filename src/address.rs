use std::fmt::Display;

use tokio::sync::oneshot;

use crate::{
    Actor, ActorError, BoxedMessageHandler, Handler, MailboxSender, Message, Result,
    handler::{Envelope, SysPoisonMessage, SystemEnvelope, SystemHandler},
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
        let envelope = Envelope::new(msg, None);
        self.send(Box::new(envelope)).await?;
        Ok(())
    }

    pub async fn ask<M>(&self, msg: M) -> Result<M::Response>
    where
        M: Message,
        A: Handler<M>,
    {
        let (reply_sender, reply_receiver) = oneshot::channel();
        let envelope = Envelope::new(msg, Some(reply_sender));
        self.send(Box::new(envelope)).await?;
        reply_receiver
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))
    }

    pub async fn poison(&self) -> Result<()> {
        self.sys_tell(SysPoisonMessage).await
    }

    pub(crate) async fn sys_tell<M>(&self, msg: M) -> Result<()>
    where
        M: Message,
        A: SystemHandler<M>,
    {
        let envelope = SystemEnvelope::new(msg, None);
        self.send(Box::new(envelope)).await?;
        Ok(())
    }

    async fn send(&self, msg: BoxedMessageHandler<A>) -> Result<()> {
        self.sender
            .inner
            .send(msg)
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))?;

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.sender.inner.is_closed()
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
