mod abi;

use prost::bytes::{Bytes, BytesMut};
use prost::Message;
pub use abi::*;


impl Request {
    pub fn new_get(key: &str) -> Self {
        Request {
            command: Some(request::Command::Get(RequestGet { key: key.to_owned() }))
        }
    }

    pub fn new_put(key: &str, value: &[u8]) -> Self {
        Request {
            command: Some(request::Command::Put(ResponsePut { key: key.to_owned(), value: value.to_vec() }))
        }
    }
}

impl TryFrom<BytesMut> for Request {
    type Error = prost::DecodeError;

    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Message::decode(value)
    }
}

impl Response {
    pub fn new(key: String, value: Vec<u8>) -> Self {
        Self {
            code: 0,
            key,
            value,
        }
    }

    pub fn not_found(key: String) -> Self {
        Self {
            code: 404,
            key,
            ..Default::default()
        }
    }
}

impl From<Response> for Bytes {
    fn from(message: Response) -> Self {
        let mut buf = BytesMut::new();
        message.encode(&mut buf).unwrap();
        buf.freeze()
    }
}

impl From<Request> for Bytes {
    fn from(message: Request) -> Self {
        let mut buf = BytesMut::new();
        message.encode(&mut buf).unwrap();
        buf.freeze()
    }
}

impl TryFrom<BytesMut> for Response {
    type Error = prost::DecodeError;

    fn try_from(value: BytesMut) -> Result<Self, Self::Error> {
        Message::decode(value)
    }
}