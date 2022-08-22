use crate::data::message::{OneBotMessageEvent};
use crate::data::{BotSelfData};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OneBotEvent {
    pub id: String,
    pub time: f64,
    #[serde(flatten)]
    pub inner: OneBotTypedEvent,
    pub sub_type: String,
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_self: Option<BotSelfData>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OneBotTypedEvent {
    Meta(OneBotMetaEvent),
    Notice,
    Message(OneBotMessageEvent),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum OneBotMetaEvent {
    Heartbeat { interval: i64 },
    StatusUpdate { status: OneBotMetaStatus, },
}

#[derive(Default,Serialize, Deserialize)]
pub struct OneBotMetaStatus {
    pub good: bool,
    pub bots: Vec<BotStatus>,
}

#[derive(Default,Serialize, Deserialize)]
pub struct BotStatus {
    #[serde(rename = "self")]
    pub bot_self: BotSelfData,
    pub online: bool,
    #[serde(flatten)]
    pub ext: Option<BotStatusExt>,
}



#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum BotStatusExt {
    #[serde(rename = "qq.status")]
    QQStatus(String), // for what?
}