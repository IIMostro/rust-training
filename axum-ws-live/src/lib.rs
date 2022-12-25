use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::Extension;
use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use dashmap::{DashMap, DashSet};
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast::{channel, Sender};
use tracing::warn;

pub use msg::{Msg, MsgData};

mod msg;

const CAPACITY: usize = 64;

struct State {
    // 用户下面有多少的房间
    user_rooms: DashMap<String, DashSet<String>>,
    // 房间下的用户数量
    rooms_users: DashMap<String, DashSet<String>>,
    // 使用arc封装，避免字节的复制
    tx: Sender<Arc<Msg>>,
}

impl State {
    fn new() -> Self {
        // broadcast::channel函数返回一个Sender和一个Receiver
        // 只需要tx.subscribe()就可以订阅消息, 并且返回一个rx,所以这里只需要一个tx就可以。
        let (tx, _) = channel(CAPACITY);
        Self {
            user_rooms: DashMap::new(),
            rooms_users: DashMap::new(),
            tx,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

// 对于只有一个字段的struct可以使用newtype模式
#[derive(Clone, Default)]
pub struct ChatState(Arc<State>);

impl ChatState {
    fn new() -> Self {
        Self(Arc::new(State::new()))
    }

    pub fn get_user_rooms(&self, user: &str) -> Vec<String> {
        self.0.user_rooms.get(user)
            .map(|room| { room.clone().into_iter().collect()
        }).unwrap_or_default()
    }

    pub fn get_room_users(&self, room: &str) -> Vec<String> {
        self.0.rooms_users.get(room)
            .map(|user| { user.clone().into_iter().collect()
        }).unwrap_or_default()
    }
}


pub async fn ws_handler(ws: WebSocketUpgrade, Extension(state): Extension<ChatState>) -> impl IntoResponse {
    ws.on_upgrade(|websocket| handle_socket(websocket, state))
}

pub async fn handle_socket(socket: WebSocket, state: ChatState) {
    let mut rx = state.0.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();
    let state1 = state.clone();

    // 处理连接
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(data)) = receiver.next().await {
            match data {
                Message::Text(msg) => {
                    handle_message(msg.as_str().try_into().unwrap(), state1.0.clone()).await;
                }
                _ => {}
            }
        }
    });

    // 处理响应
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let data = msg.as_ref().try_into().unwrap();
            if sender.send(Message::Text(data)).await.is_err() {
                warn!("websocket send error");
                break;
            }
        }
    });


    tokio::select! {
        _ = &mut recv_task => send_task.abort(),
        _ = &mut send_task => recv_task.abort()
    }

    warn!("websocket closed");

}

async fn handle_message(msg: Msg, state: Arc<State>) {
    let msg = match msg.data {
        MsgData::Join => {
            let username = msg.username.clone();
            let room = msg.room.clone();
            state.user_rooms.entry(username.clone()).or_default().insert(room.clone());
            state.rooms_users.entry(room).or_default().insert(username);
            msg
        }

        MsgData::Leave => {
            if let Some(v) = state.user_rooms.get_mut(&msg.username) {
                v.remove(&msg.room);
                if v.is_empty() {
                    state.user_rooms.remove(&msg.username);
                }
            }
            if let Some(v) = state.rooms_users.get_mut(&msg.room) {
                v.remove(&msg.username);
                if v.is_empty() {
                    state.rooms_users.remove(&msg.room);
                }
            }
            msg
        }
        _ => msg
    };
    if let Err(ex) = state.tx.send(Arc::new(msg)) {
        warn!("send msg error: {ex}");
    }
}