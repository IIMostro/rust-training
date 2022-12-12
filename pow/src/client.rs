use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use protobuf::{*, pow_builder_client::*};

mod protobuf;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = "http://127.0.0.1:8888";
    let mut client = PowBuilderClient::connect(addr).await?;
    info!("pow client listening on {:?}", addr);
    // 首先订阅，订阅成功后，会返回一个channel，用于接收pow engine返回的数据
    let mut stream = client.subscribe(ClientInfo {
        name: "client2".to_string(),
    }).await?.into_inner();
    info!("client1 subscribe success!");
    let res = client.submit(Block {
        data: b"hello world".to_vec()
    }).await?.into_inner();
    info!("client1 submit block success! {:?}", res);
    while let Some(result) = stream.message().await? {
        info!("result id: {:?}, hash:{:?}, nonce:{:?}", hex::encode(result.id), hex::encode(result.hash), result.nonce);
    }

    Ok(())
}