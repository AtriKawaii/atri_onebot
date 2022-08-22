use crate::data::message::{OneBotMessage, OneBotMessageEvent};
use crate::data::BotSelfData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OneBotEvent {
    pub id: String,
    pub time: f64,
    #[serde(flatten)]
    pub inner: OneBotEventInner,
    pub sub_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct OneBotSelfEvent {
    #[serde(flatten)]
    pub inner: OneBotEvent,
    #[serde(rename = "self")]
    pub bot_self: BotSelfData,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OneBotEventInner {
    Meta(OneBotMetaEvent),
    Notice,
    Message(OneBotMessageEvent),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "lowercase")]
pub enum OneBotMetaEvent {
    HeartBeat { interval: i64 },
}
