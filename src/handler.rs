use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::oneshot;

use crate::{Actor, ActorContext, Handler, Message};

pub enum MessageHandlerResult {
    None,
    Stop { reason: String },
    Timeout,
}

#[async_trait]
pub trait MessageHandler<A: Actor>: Send + Sync {
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext) -> MessageHandlerResult;
}

pub type BoxedMessageHandler<A> = Box<dyn MessageHandler<A>>;

#[derive(Debug)]
pub struct Envelope<M, A>
where
    M: Message,
    A: Handler<M>,
{
    payload: Option<M>,
    reply_to: Option<oneshot::Sender<M::Response>>,
    _actor: PhantomData<A>,
}

impl<M, A> Envelope<M, A>
where
    M: Message,
    A: Handler<M>,
{
    pub fn new(msg: M, reply_to: Option<oneshot::Sender<M::Response>>) -> Self {
        Envelope {
            payload: Some(msg),
            reply_to,
            _actor: PhantomData,
        }
    }
}

#[async_trait]
impl<M, A> MessageHandler<A> for Envelope<M, A>
where
    M: Message,
    A: Handler<M>,
{
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext) -> MessageHandlerResult {
        let result = actor.handle(self.payload.take().unwrap(), ctx).await;

        if let Some(reply_to) = self.reply_to.take() {
            reply_to.send(result).unwrap_or_else(|_failed| {
                eprintln!("Failed to send back response!");
            })
        }

        MessageHandlerResult::None
    }
}

#[async_trait]
pub trait SystemHandler<M: Message>: Actor {
    async fn handle(
        &mut self,
        msg: M,
        ctx: &mut ActorContext,
    ) -> (M::Response, MessageHandlerResult);
}

#[derive(Debug)]
pub struct SystemEnvelope<M, A>
where
    M: Message,
    A: SystemHandler<M>,
{
    payload: Option<M>,
    reply_to: Option<oneshot::Sender<M::Response>>,
    _actor: PhantomData<A>,
}

impl<M, A> SystemEnvelope<M, A>
where
    M: Message,
    A: SystemHandler<M>,
{
    pub fn new(msg: M, reply_to: Option<oneshot::Sender<M::Response>>) -> Self {
        SystemEnvelope {
            payload: Some(msg),
            reply_to,
            _actor: PhantomData,
        }
    }
}

#[async_trait]
impl<M, A> MessageHandler<A> for SystemEnvelope<M, A>
where
    M: Message,
    A: SystemHandler<M>,
{
    async fn handle(&mut self, actor: &mut A, ctx: &mut ActorContext) -> MessageHandlerResult {
        let (result, handler_result) = actor.handle(self.payload.take().unwrap(), ctx).await;

        if let Some(reply_to) = self.reply_to.take() {
            reply_to.send(result).unwrap_or_else(|_failed| {
                eprintln!("Failed to send back response!");
            })
        }

        handler_result
    }
}

#[derive(Debug)]
pub struct PoisonMessage;

impl Message for PoisonMessage {
    type Response = ();
}

#[async_trait]
impl<A: Actor> SystemHandler<PoisonMessage> for A {
    async fn handle(
        &mut self,
        _msg: PoisonMessage,
        _ctx: &mut ActorContext,
    ) -> ((), MessageHandlerResult) {
        (
            (),
            MessageHandlerResult::Stop {
                reason: "poisoned".into(),
            },
        )
    }
}
