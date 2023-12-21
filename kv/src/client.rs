mod protobuf;
mod noise_codec;

use protobuf::*;
use prost::Message;
use anyhow::Result;
use tokio_util::codec::LengthDelimitedCodec;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use futures::{StreamExt, SinkExt};


#[tokio::main]
async fn main() -> Result<()>{
    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::from(Level::INFO))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let addr = "localhost:8888";
    let stream = tokio::net::TcpStream::connect(addr).await?;
    let mut stream = LengthDelimitedCodec::builder().length_field_length(2)
        .new_framed(stream);
    let request = Request::new_put("hello", b"world");
    stream.send(request.into()).await?;
    let request = Request::new_get("hello");
    stream.send(request.into()).await?;
    while let Some(Ok(buf)) = stream.next().await {
        // Response::try_from(buf)?;
        let response:Response = buf.try_into()?;
        info!("response:[{:?}]", response);
    }
    Ok(())
}
