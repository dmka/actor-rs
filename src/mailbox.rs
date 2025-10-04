use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::{mpsc, oneshot};

pub use crate::address::{ActorPath, ActorRef};
pub use crate::handler::{BoxedMessageHandler, MessageHandler, MessageHandlerResult};
use crate::handler::{SystemEnvelope, SystemHandler};
use crate::{Message, Result, handler::Envelope};

use crate::{Actor, ActorContext, ActorError, Handler};

#[derive(Debug)]
pub struct MailboxReceiver<A: Actor> {
    inner: mpsc::Receiver<BoxedMessageHandler<A>>,
}

#[derive(Debug)]
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

impl<A: Actor> MailboxSender<A> {
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
        let (response_sender, response_receiver) = oneshot::channel();
        let envelope = Envelope::new(msg, Some(response_sender));
        self.send(Box::new(envelope)).await?;
        response_receiver
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))
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
        self.inner
            .send(msg)
            .await
            .map_err(|e| ActorError::SendError(e.to_string()))?;

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

#[async_trait]
pub trait MessageProcessor<A: Actor> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A);
}

pub trait Mailbox<A: Actor>: MessageProcessor<A> + Send + 'static {
    fn sender(&mut self) -> MailboxSender<A>;
}

#[derive(Debug)]
pub struct DefaultMailbox<A: Actor> {
    sender: Option<MailboxSender<A>>,
    receiver: MailboxReceiver<A>,
    _actor: PhantomData<A>,
}

impl<A: Actor> DefaultMailbox<A> {
    pub fn new(buffer: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer);
        Self {
            sender: Some(MailboxSender { inner: sender }),
            receiver: MailboxReceiver { inner: receiver },
            _actor: PhantomData,
        }
    }
}

impl<A: Actor> Mailbox<A> for DefaultMailbox<A> {
    fn sender(&mut self) -> MailboxSender<A> {
        self.sender.take().unwrap()
    }
}

#[async_trait]
impl<A: Actor> MessageProcessor<A> for DefaultMailbox<A> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A) {
        while let Some(mut msg) = self.receiver.inner.recv().await {
            match msg.handle(actor, ctx).await {
                MessageHandlerResult::Stop { reason } => {
                    println!("stop: reason={reason}");
                    self.receiver.inner.close();
                    ctx.system.stop_actor(&ctx.path).await;
                    break;
                }
                MessageHandlerResult::Timeout => {}
                MessageHandlerResult::None => {}
            }
        }
    }
}
