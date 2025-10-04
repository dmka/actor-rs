use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::mpsc;

pub use crate::address::{ActorPath, ActorRef};
pub use crate::handler::{BoxedMessageHandler, MessageHandler, MessageHandlerResult};

use crate::{Actor, ActorContext};

pub type MailboxReceiver<A: Actor> = mpsc::Receiver<BoxedMessageHandler<A>>;
pub type MailboxSender<A: Actor> = mpsc::Sender<BoxedMessageHandler<A>>;

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
            sender: Some(sender),
            receiver,
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
        while let Some(mut msg) = self.receiver.recv().await {
            match msg.handle(actor, ctx).await {
                MessageHandlerResult::Stop { reason } => {
                    println!("stop: reason={reason}");
                    self.receiver.close();
                    ctx.system.stop_actor(&ctx.path).await;
                    break;
                }
                MessageHandlerResult::Timeout => {}
                MessageHandlerResult::None => {}
            }
        }
    }
}
