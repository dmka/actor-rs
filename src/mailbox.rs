use async_trait::async_trait;
use std::marker::PhantomData;
use tokio::sync::mpsc;

pub use crate::handler::{BoxedMessageHandler, MessageHandler, MessageHandlerResult};
pub use crate::reference::{ActorPath, ActorRef};

use crate::{Actor, ActorContext};

pub type Receiver<A> = mpsc::Receiver<BoxedMessageHandler<A>>;
pub type Sender<A> = mpsc::Sender<BoxedMessageHandler<A>>;
pub type WeakSender<A> = mpsc::WeakSender<BoxedMessageHandler<A>>;

#[async_trait]
pub trait MessageProcessor<A: Actor> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A);
}

pub trait Mailbox<A: Actor>: MessageProcessor<A> + Send + 'static {
    fn take_sender(&mut self) -> Sender<A>;
}

#[derive(Debug)]
pub struct DefaultMailbox<A: Actor> {
    sender: Option<Sender<A>>,
    receiver: Receiver<A>,
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
    fn take_sender(&mut self) -> Sender<A> {
        self.sender.take().unwrap()
    }
}

#[async_trait]
impl<A: Actor> MessageProcessor<A> for DefaultMailbox<A> {
    async fn process_messages(&mut self, ctx: &mut ActorContext, actor: &mut A) {
        while let Some(mut msg) = self.receiver.recv().await {
            match msg.handle(actor, ctx).await {
                MessageHandlerResult::Stop { reason } => match actor.stopping(ctx, &reason).await {
                    crate::StoppingResult::Stop => {
                        println!("stop: reason={reason}");
                        self.receiver.close();
                        break;
                    }
                    crate::StoppingResult::Cancel => {}
                },
                MessageHandlerResult::Timeout => {}
                MessageHandlerResult::None => {}
            }
        }
    }
}
