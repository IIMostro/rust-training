extern crate redis;

use std::sync::Arc;
use std::thread;
use anyhow::Result;

use redis::{Client, Commands, ControlFlow, PubSubCommands};

trait AppState {
    fn client(&self) -> &Arc<Client>;
}

struct Ctx {
    pub client: Arc<Client>,
}

impl Ctx {
    fn new() -> Ctx {
        let client = Client::open("redis://172.29.10.180/").unwrap();
        Ctx {
            client: Arc::new(client)
        }
    }
}

impl AppState for Ctx {
    fn client(&self) -> &Arc<Client> {
        &self.client
    }
}

fn subscribe(state: &impl AppState) -> thread::JoinHandle<()> {
    let client = Arc::clone(state.client());
    let mut conn = client.get_connection().unwrap();
    thread::spawn(move || {
        conn.subscribe(&["my_rust_test_channel"], |msg| {
            let channel_name = msg.get_channel_name();
            let payload: String = msg.get_payload().unwrap();
            println!("channel_name: {}, payload: {}", channel_name, payload);
            ControlFlow::<String>::Continue
        }).unwrap();
    })
}
#[tokio::main]
async fn main() -> Result<()>{
    let ctx = Ctx::new();
    let status = subscribe(&ctx);
    let status1= subscribe(&ctx);
    println!("subscribed!");
    let client = Arc::clone(ctx.client());
    let mut conn = client.get_connection().unwrap();
    let _:() = conn.publish("my_rust_test_channel", "hello world").unwrap();
    status.join().unwrap();
    status1.join().unwrap();
    Ok(())
}
