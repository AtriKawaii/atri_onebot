pub mod event;
pub mod message;
pub mod notice;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use atri_plugin::bot::Bot;

#[derive(Serialize, Deserialize)]
pub struct ActionResponse {
    pub status: ActionStatus,
    pub retcode: i64,
    pub data: Value,
    pub message: String,
}

impl ActionResponse {
    pub fn from_result<T: Serialize, E: Error>(result: Result<T, E>, code: i64) -> Self {
        match result {
            Ok(t) => ActionResponse {
                status: ActionStatus::Ok,
                retcode: 0,
                data: serde_json::to_value(t).unwrap(),
                message: "".to_string(),
            },
            Err(e) => ActionResponse {
                status: ActionStatus::Failed,
                retcode: code,
                data: Value::Null,
                message: e.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Ok,
    Failed,
}

#[derive(Default,Serialize, Deserialize)]
pub struct BotSelfData {
    pub platform: String,
    pub user_id: String,
}

impl From<Bot> for BotSelfData {
    fn from(bot: Bot) -> Self {
        Self {
            platform: "qq".to_string(),
            user_id: bot.id().to_string(),
        }
    }
}