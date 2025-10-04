mod actor;

use async_trait::async_trait;

use crate::actor::{Actor, ActorContext, ActorError, ActorSystem, Handler, Message};

#[derive(Default)]
struct TestActor {
    counter: usize,
}

#[async_trait]
impl Actor for TestActor {
    async fn pre_start(&mut self, _ctx: &mut ActorContext) -> Result<(), ActorError> {
        println!("Starting actor TestActor!");
        Ok(())
    }

    async fn post_stop(&mut self, _ctx: &mut ActorContext) {
        println!("Stopped actor TestActor!");
    }
}

#[derive(Debug)]
struct TestMessage(usize);

impl Message for TestMessage {
    type Response = usize;
}

impl Drop for TestMessage {
    fn drop(&mut self) {
        println!("->> Msg Dropped: {self:?}");
    }
}

#[async_trait]
impl Handler<TestMessage> for TestActor {
    async fn handle(&mut self, msg: TestMessage, _ctx: &mut ActorContext) -> usize {
        self.counter += 1;
        println!("->> WORKS!");
        msg.0
    }
}

#[tokio::main]
async fn main() {
    //let mut actor = MyActorRef::spawn().unwrap();

    // for _ in 0..3 {
    //     let c = actor.get_count().await.unwrap();
    //     println!("{c}");
    // }

    // actor.shutdown().await.unwrap();

    let sys = ActorSystem::new("main");

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
}
