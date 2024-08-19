use anyhow::Result;
use etcd_rs::{Client, ClientConfig, KeyRange, KeyValueOp, WatchInbound, WatchOp};
use serde::{Deserialize, Serialize};
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::connect(ClientConfig::new(["http://192.168.205.10:2379".into()])).await?;
    let values = client.get_all().await?;
    for kv in values.kvs {
        println!("{}: {}", kv.key_str(), kv.value_str());
    }
    watch(client).await?;
    Ok(())
}

async fn watch(client: Client) -> Result<()> {
    let (mut stream, _) = client.watch(KeyRange::prefix("test_user")).await?;
    // tokio::spawn(async move {
    //     tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    //     let user = User { username: "neptune".to_string(), age: 26 };
    //     let value = to_string(&user).unwrap();
    //     client.put(("test_user", value)).await.unwrap();
    // });
    while let event = stream.inbound().await {
        match event {
            WatchInbound::Ready(resp) => {
                for event in resp.events {
                    println!("event: type: {:?}, key:{}, value:{}", event.event_type, event.kv.key_str(), event.kv.value_str());
                }
                println!("watch ready: {}", resp.watch_id);
            }
            WatchInbound::Interrupted(_) => {
                println!("watch error");
                break;
            }
            WatchInbound::Closed => {
                println!("watch was closed");
                break;
            }
        }
    };
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    username: String,
    age: u32,
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use etcd_rs::{Client, ClientConfig, KeyValueOp};
    use serde_json::to_string;

    use crate::User;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_save_should_work() -> Result<()> {
        let client = Client::connect(ClientConfig::new(["http://192.168.205.10:2379".into()])).await?;
        let user = User { username: "neptune".to_string(), age: 26 };
        client.put(("test_user", to_string(&user).unwrap())).await?;
        Ok(())
    }
}
