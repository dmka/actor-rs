use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::{mpsc, oneshot};

use crate::actor::{Actor, ActorContext, ActorError, ActorPath, Handler, Message};

#[async_trait]
pub trait MessageHandler<A: Actor>: Send + Sync {
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext);
}

pub type BoxedMessageHandler<A> = Box<dyn MessageHandler<A>>;

pub struct MailboxReceiver<A: Actor> {
    inner: mpsc::Receiver<BoxedMessageHandler<A>>,
}

pub struct MailboxSender<A: Actor> {
    inner: mpsc::Sender<BoxedMessageHandler<A>>,
}

impl<A: Actor> Clone for MailboxSender<A> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[async_trait]
pub trait MessageProcessor<A: Actor> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A);
}

pub trait Mailbox<A: Actor>: MessageProcessor<A> + Send + Sync + 'static {
    fn sender(&self) -> MailboxSender<A>;
}

pub struct DefaultMailbox<A: Actor> {
    sender: MailboxSender<A>,
    receiver: MailboxReceiver<A>,
    _actor: PhantomData<A>,
}

impl<A: Actor> DefaultMailbox<A> {
    pub fn new(buffer: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer);
        Self {
            sender: MailboxSender { inner: sender },
            receiver: MailboxReceiver { inner: receiver },
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> Mailbox<A> for DefaultMailbox<A> {
    fn sender(&self) -> MailboxSender<A> {
        self.sender.clone()
    }
}

#[async_trait]
impl<A: Actor> MessageProcessor<A> for DefaultMailbox<A> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A) {
        while let Some(mut msg) = self.receiver.inner.recv().await {
            msg.handle(actor, ctx).await;
        }
    }
}

pub struct ActorMessage<M, A>
where
    M: Message,
    A: Handler<M>,
{
    payload: Option<M>,
    reply_to: Option<oneshot::Sender<M::Response>>,
    _actor: PhantomData<A>,
}

impl<M, A> ActorMessage<M, A>
where
    M: Message,
    A: Handler<M>,
{
    fn new(msg: M, reply_to: Option<oneshot::Sender<M::Response>>) -> Self {
        ActorMessage {
            payload: Some(msg),
            reply_to,
            _actor: PhantomData,
        }
    }
}

#[async_trait]
impl<M, A> MessageHandler<A> for ActorMessage<M, A>
where
    M: Message,
    A: Handler<M>,
{
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext) {
        let result = actor.handle(self.payload.take().unwrap(), ctx).await;

        if let Some(reply_to) = self.reply_to.take() {
            reply_to.send(result).unwrap_or_else(|_failed| {
                eprintln!("Failed to send back response!");
            })
        }
    }
}

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

    pub async fn tell<M>(&self, msg: M) -> Result<(), ActorError>
    where
        M: Message,
        A: Handler<M>,
    {
        let message = ActorMessage::new(msg, None);
        if let Err(error) = self.sender.inner.send(Box::new(message)).await {
            eprintln!("Failed to tell message! {}", error);
            Err(ActorError::SendError(error.to_string()))
        } else {
            Ok(())
        }
    }

    pub async fn ask<M>(&self, msg: M) -> Result<M::Response, ActorError>
    where
        M: Message,
        A: Handler<M>,
    {
        let (response_sender, response_receiver) = oneshot::channel();
        let message = ActorMessage::new(msg, Some(response_sender));
        if let Err(error) = self.sender.inner.send(Box::new(message)).await {
            eprintln!("Failed to ask message! {}", error);
            Err(ActorError::SendError(error.to_string()))
        } else {
            response_receiver
                .await
                .map_err(|error| ActorError::SendError(error.to_string()))
        }
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
