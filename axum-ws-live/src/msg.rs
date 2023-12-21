use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Msg{
    pub room: String,
    pub username: String,
    pub timestamp: u64,
    pub data: MsgData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MsgData{
    Join, Leave, Message(String)
}

// 反序列化
impl TryFrom<&str> for Msg{
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

// 序列化
impl TryFrom<&Msg> for String{
    type Error = serde_json::Error;

    fn try_from(value: &Msg) -> Result<Self, Self::Error> {
        serde_json::to_string(value)
    }
}

impl Msg{
    pub fn new(room: String, username: String, data: MsgData) -> Self{
        Msg{
            room,
            username,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            data,
        }
    }

    pub fn join(room: &str, username: &str) -> Self{
        Msg::new(room.into(), username.into(), MsgData::Join)
    }

    pub fn leave(room: &str, username: &str) -> Self{
        Msg::new(room.into(), username.into(), MsgData::Leave)
    }

    pub fn message(room: &str, username: &str, message: &str) -> Self{
        Msg::new(room.into(), username.into(), MsgData::Message(message.into()))
    }
}

