pub mod event;
pub mod message;
pub mod notice;

use crate::data::event::OneBotMetaStatus;
use atri_plugin::bot::Bot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct ActionResponse {
    pub status: ActionStatus,
    pub retcode: i64,
    pub data: Option<ActionData>,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionData {
    GetStatus(OneBotMetaStatus),
}

impl ActionResponse {
    pub fn from_err<E: Error>(err: E, code: i64) -> Self {
        Self {
            status: ActionStatus::Failed,
            retcode: code,
            data: None,
            message: err.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Ok,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub struct ActionRequest {
    action: String,
    params: Value,
}

#[derive(Clone, Default, Serialize, Deserialize)]
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
