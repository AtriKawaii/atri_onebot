use crate::data::action::BotData;
use crate::data::message::OneBotMessageEvent;
use atri_plugin::bot::Bot;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct OneBotEvent {
    pub id: String,
    pub time: f64,
    #[serde(flatten)]
    pub inner: OneBotTypedEvent,
    pub sub_type: &'static str,
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_self: Option<BotData>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum OneBotTypedEvent {
    Meta(OneBotMetaEvent),
    Notice,
    Message(OneBotMessageEvent),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum OneBotMetaEvent {
    Heartbeat { interval: i64 },
    StatusUpdate { status: OneBotMetaStatus },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OneBotMetaStatus {
    pub good: bool,
    pub bots: Vec<BotStatus>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BotStatus {
    #[serde(rename = "self")]
    pub bot_self: BotData,
    pub online: bool,
    #[serde(flatten)]
    pub ext: Option<BotStatusExt>,
}

impl From<Bot> for BotStatus {
    fn from(bt: Bot) -> Self {
        Self {
            bot_self: bt.into(),
            online: true,
            ext: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BotStatusExt {
    #[serde(rename = "qq.status")]
    QQStatus(String), // for what?
}
