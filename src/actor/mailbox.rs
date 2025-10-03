use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::{mpsc, oneshot};

use crate::actor::{Actor, ActorContext, ActorError, ActorPath, Handler, Message};

#[async_trait]
pub trait MessageHandler<A: Actor>: Send + Sync {
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext);
}

type BoxedMessageHandler<A> = Box<dyn MessageHandler<A>>;
pub type MailboxReceiver<A> = mpsc::UnboundedReceiver<BoxedMessageHandler<A>>;
type MailboxSender<A> = mpsc::UnboundedSender<BoxedMessageHandler<A>>;

pub struct DefaultMailbox<A: Actor> {
    sender: MailboxSender<A>,
    receiver: MailboxReceiver<A>,
    _actor: PhantomData<A>,
}

pub trait MessageProcessor<A: Actor> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A);
}

impl<A: Actor> DefaultMailbox<A> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver,
            _actor: PhantomData,
        }
    }

    pub fn sender(&self) -> MailboxSender<A> {
        self.sender.clone()
    }
}

impl<A: Actor> MessageProcessor<A> for DefaultMailbox<A> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A) {
        while let Some(mut msg) = self.receiver.recv().await {
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

        if let Some(rsvp) = self.reply_to.take() {
            rsvp.send(result).unwrap_or_else(|_failed| {
                eprintln!("Failed to send back response!");
            })
        }
    }
}

pub struct ActorRef<A: Actor> {
    path: ActorPath,
    sender: mpsc::UnboundedSender<BoxedMessageHandler<A>>,
}

impl<A: Actor> ActorRef<A> {
    pub fn new(path: ActorPath, sender: MailboxSender<A>) -> Self {
        ActorRef { path, sender }
    }

    pub fn path(&self) -> &ActorPath {
        &self.path
    }

    pub fn tell<M>(&self, msg: M) -> Result<(), ActorError>
    where
        M: Message,
        A: Handler<M>,
    {
        let message = ActorMessage::new(msg, None);
        if let Err(error) = self.sender.send(Box::new(message)) {
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
        if let Err(error) = self.sender.send(Box::new(message)) {
            eprintln!("Failed to ask message! {}", error);
            Err(ActorError::SendError(error.to_string()))
        } else {
            response_receiver
                .await
                .map_err(|error| ActorError::SendError(error.to_string()))
        }
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
