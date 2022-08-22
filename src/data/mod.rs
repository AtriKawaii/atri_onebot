pub mod event;
pub mod message;
pub mod notice;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Response {
    pub status: Status,
    pub retcode: i64,
    pub data: Value,
    pub message: String,
}

impl Response {
    pub fn from_result<T: Serialize, E: Error>(result: Result<T, E>, code: i64) -> Self {
        match result {
            Ok(t) => Response {
                status: Status::Ok,
                retcode: 0,
                data: serde_json::to_value(t).unwrap(),
                message: "".to_string(),
            },
            Err(e) => Response {
                status: Status::Failed,
                retcode: code,
                data: Value::Null,
                message: e.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Serialize, Deserialize)]
pub struct BotSelfData {
    pub platform: String,
    pub user_id: String,
}
