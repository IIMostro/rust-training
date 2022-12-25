use std::net::SocketAddr;
use axum::{Extension, Router, Server};
use axum::handler::Handler;
use axum::routing::get;
use axum_ws_live::{ChatState, ws_handler};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ws", get(ws_handler).layer(Extension(ChatState::default())));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
