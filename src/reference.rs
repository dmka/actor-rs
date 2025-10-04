use std::{fmt::Display, ops::Deref};

use tokio::sync::oneshot;

use crate::{
    Actor, ActorError, BoxedMessageHandler, Handler, Message, Result, Sender,
    handler::{Envelope, SystemEnvelope, SystemHandler},
    system::SystemMessage,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ActorPath {
    inner: String,
}

impl ActorPath {
    pub fn new<T: Into<String>>(path: T) -> Self {
        Self { inner: path.into() }
    }

    pub fn name(&self) -> &str {
        let parts: Vec<&str> = self.inner.split('/').collect();
        parts.last().unwrap()
    }

    pub fn has_parent(&self) -> bool {
        self.inner.contains("/")
    }

    pub fn parent(&self) -> Option<ActorPath> {
        let parts: Vec<&str> = self.inner.split('/').collect();
        if parts.len() > 1 {
            let inner = parts[0..parts.len() - 1].join("/");
            Some(ActorPath { inner })
        } else {
            None
        }
    }

    pub fn join<T: Into<String>>(&self, path: T) -> ActorPath {
        let new_path = format!("{}/{}", self.as_ref(), path.into());
        ActorPath::new(new_path)
    }
}

impl Display for ActorPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<String> for ActorPath {
    #[inline]
    fn as_ref(&self) -> &String {
        &self.inner
    }
}

impl Deref for ActorPath {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct ActorRef<A: Actor> {
    path: ActorPath,
    sender: Sender<A>,
}

impl<A: Actor> ActorRef<A> {
    pub fn new(path: ActorPath, sender: Sender<A>) -> Self {
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
        let _ = self.sys_ask(SystemMessage::Poison).await;
        Ok(())
    }

    pub(crate) async fn sys_ask<M>(&self, msg: M) -> Result<M::Response>
    where
        M: Message,
        A: SystemHandler<M>,
    {
        let (reply_sender, reply_receiver) = oneshot::channel();
        let envelope = SystemEnvelope::new(msg, Some(reply_sender));
        self.send(Box::new(envelope)).await?;
        reply_receiver
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))
    }

    #[allow(dead_code)]
    pub(crate) async fn sys_tell<M>(&self, msg: M) -> Result<()>
    where
        M: Message,
        A: SystemHandler<M>,
    {
        let envelope = SystemEnvelope::new(msg, None);
        self.send(Box::new(envelope)).await?;
        Ok(())
    }

    #[inline]
    async fn send(&self, msg: BoxedMessageHandler<A>) -> Result<()> {
        self.sender
            .send(msg)
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))?;

        Ok(())
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
