mod protobuf;
mod pow;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc};
use std::thread;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::Sender;
use tonic::{Request, Response, Status};
use futures::Stream;
use tokio_stream::wrappers::ReceiverStream;
use protobuf::*;
use anyhow::Result;
use tonic::transport::Server;
use tracing::{error, info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use pow::*;
use crate::pow_builder_server::{PowBuilder, PowBuilderServer};

const CHANNEL_SIZE: usize = 8;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:8888";
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("pow server listening on {}", addr);
    start_server(addr).await?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
struct Share {
    clients: HashMap<String, Sender<Result<BlockHash, Status>>>,
}

impl Share {

    // 给每个client发送消息
    async fn broadcast(&self, message: Option<BlockHash>) {
        let msg = message.ok_or(Status::resource_exhausted("broadcast error"));
        for (name, tx) in &self.clients {
            match tx.send(Ok(msg.as_ref().unwrap().clone())).await {
                Ok(_) => info!("send message to client:[{}] success!", name),
                Err(_) => error!("send message to client:[{}] error!", name)
            }
        }
    }
}

pub struct PowService {
    // 通道，pow接收到新的区块后，会将区块hash发送到pow engine
    tx: Sender<Block>,
    // 客户端，用户返回客户端信息
    shares: Arc<RwLock<Share>>,

}

impl PowService {

    // tx client发送过来的数据, rx pow engine返回的数据
    pub fn new(tx: Sender<Block>, mut rx: mpsc::Receiver<Option<BlockHash>>) -> Self {
        let service = PowService {
            tx,
            shares: Arc::new(RwLock::new(Share::default())),
        };
        let shared = service.shares.clone();
        // 创建实例的时候开启一个线程，当pow engine返回数据后，将数据发送给所有的客户端
        tokio::spawn(async move {
            while let Some(hash) = rx.recv().await {
                shared.read().await.broadcast(hash).await;
            }
        });
        service
    }
}

async fn start_server(addr: &str) -> Result<()> {
    // grpc -> Pow
    // 创建一个channel, 用户接收client过来的消息发送到pow
    let (tx1, mut rx1) = mpsc::channel(CHANNEL_SIZE);

    // Pow -> grpc
    // pow计算好了之后发送到用户的管道
    let (tx2, rx2) = mpsc::channel(CHANNEL_SIZE);

    thread::spawn(move || {
        // 线程中需要使用blocking_recv和blocking_send。因为tokio::sync::mpsc::Receiver和Sender是非阻塞的
        // 不阻塞估计是会空转
        // 接收client发送过来的消息
        // client -> pow -> client
        while let Some(block) = rx1.blocking_recv() {
            let result = pow_v2(block);
            // 计算好了之后发送给client
            tx2.blocking_send(result).unwrap();
        }
    });

    // 创建一个PowService
    let svc = PowService::new(tx1, rx2);
    Server::builder()
        .add_service(PowBuilderServer::new(svc))
        .serve(addr.parse()?)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl PowBuilder for PowService {
    type SubscribeStream = Pin<Box<dyn Stream<Item=Result<BlockHash, Status>> + Send + Sync>>;

    /// 客户端订阅，用户计算好了之后返回pow结果
    async fn subscribe(&self, request: Request<ClientInfo>) -> Result<Response<Self::SubscribeStream>, Status> {
        let name = request.into_inner().name;
        let rx = {
            let (tx, rx) = mpsc::channel(CHANNEL_SIZE);
            // 将客户端的发送通道存储起来
            self.shares.write().await.clients.insert(name, tx);
            rx
        };
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    /// 提交计算
    async fn submit(&self, request: Request<Block>) -> Result<Response<BlockStatus>, Status> {
        let block = request.into_inner();
        match self.tx.send(block.clone()).await {
            Ok(_) => Ok(Response::new(BlockStatus { status: 0 })),
            Err(err) => {
                info!("failure submit :{:?} to pow server. Error:{:?}", block, err);
                Ok(Response::new(BlockStatus { status: 500 }))
            }
        }
    }
}