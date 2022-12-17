mod protobuf;
mod noise_codec;

use std::sync::Arc;
use std::convert::TryInto;
use dashmap::DashMap;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use protobuf::*;
use protobuf::request::*;
use anyhow::Result;
use tokio_util::codec::LengthDelimitedCodec;
use tokio::net::{TcpListener};
use futures::{StreamExt, SinkExt};

#[derive(Debug)]
struct ServerState {
    state: DashMap<String, Vec<u8>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            state: DashMap::new(),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 日志初始化
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(ServerState::new());
    let addr = "0.0.0.0:8888";
    info!("Starting server in [{:?}]", addr);
    // tcp监听
    let listener = TcpListener::bind(addr).await?;
    loop {
        // 阻塞等待连接
        let (stream, socket_addr) = listener.accept().await?;
        info!("accept a new connection: [{:?} accept]", socket_addr);
        let share = state.clone();
        tokio::spawn(async move {
            // 解包
            let mut stream = LengthDelimitedCodec::builder().length_field_length(2)
                .new_framed(stream);
            while let Some(Ok(buf)) = stream.next().await {
                // 这个地方要指明类型，不然编译不通过
                let request:Request = buf.try_into()?;
                let response = match request.command {
                    Some(Command::Get(RequestGet{key})) => {
                        let value = share.state.get(&key);
                        match value {
                            None => Response::not_found(key),
                            Some(v) => {
                                Response::new(key, v.value().to_vec())
                            }
                        }
                    }
                    Some(Command::Put(ResponsePut{key, value})) => {
                        share.state.insert(key.clone(), value.clone());
                        Response::new(key, value)
                    }
                    _ => Response::default(),
                };
                stream.send(response.into()).await?;
            }
            Ok::<(), anyhow::Error>(())
        });
    }
}
