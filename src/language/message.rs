use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ClientMessage {
    Request(Request),
    Notification(Notification),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: serde_json::Value,
}

impl Request {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Request {
            jsonrpc: String::from("2.0"),
            id: (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                / i32::MAX as u128) as i32,
            method: String::from(method),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ServerMessage {
    Response(Response),
    Notification(Notification),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub jsonrpc: String,
    pub id: i32,
    pub result: serde_json::Value,
    pub error: Option<ResponseError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

impl Notification {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Notification {
            jsonrpc: String::from("2.0"),
            method: String::from(method),
            params,
        }
    }
}
