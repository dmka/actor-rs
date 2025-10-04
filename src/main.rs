mod actor;

use std::time::Duration;

use async_trait::async_trait;

use crate::actor::{Actor, ActorContext, ActorError, ActorSystem, Handler, HandlerResult, Message};

#[derive(Default)]
struct TestActor {
    counter: usize,
}

#[async_trait]
impl Actor for TestActor {
    async fn pre_start(&mut self, ctx: &mut ActorContext) -> Result<(), ActorError> {
        println!("->> starting {}", ctx.path);
        Ok(())
    }

    async fn post_stop(&mut self, ctx: &mut ActorContext) {
        println!("->> stopped {}", ctx.path);
    }
}

#[derive(Debug)]
struct TestMessage(usize);

impl Message for TestMessage {
    type Response = usize;
}

#[async_trait]
impl Handler<TestMessage> for TestActor {
    async fn handle(
        &mut self,
        msg: TestMessage,
        _ctx: &mut ActorContext,
    ) -> (usize, HandlerResult) {
        self.counter += 1;
        println!("->> WORKS!");
        (msg.0, HandlerResult::None)
    }
}

#[tokio::main]
async fn main() {
    let sys = ActorSystem::new("main");
    {
        let a1 = sys
            .spawn_actor("a1", TestActor { counter: 0 }, 100)
            .await
            .unwrap();

        let a2 = sys
            .spawn_actor("a2", TestActor { counter: 0 }, 100)
            .await
            .unwrap();

        let r = a1.ask(TestMessage(7)).await.unwrap();
        dbg!(r);
        let r = a2.ask(TestMessage(5)).await.unwrap();
        dbg!(r);

        a2.poison().await.unwrap();

        // sys.stop_actor(a1.path()).await;
        // sys.stop_actor(a2.path()).await;
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
}
