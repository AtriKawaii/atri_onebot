use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "detail_type")]
pub enum OneBotMessageEvent {
    Private {
        #[serde(flatten)]
        message: OneBotMessage,
        user_id: String,
    },
    Group {
        #[serde(flatten)]
        message: OneBotMessage,
        group_id: String,
    },
}

#[derive(Serialize, Deserialize)]
pub struct OneBotMessage {
    pub message_id: String,
    pub message: Vec<MessageElement>,
    pub alt_message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum MessageElement {
    Text {
        text: String,
    },
    Image {
        file_id: String,
    },
    Mention {
        user_id: String,
    },
    MentionAll {},
    Voice {
        file_id: String,
    },
    Audio {
        file_id: String,
    },
    Video {
        file_id: String,
    },
    File {
        file_id: String,
    },
    Location {
        /// 纬度
        latitude: f64,
        /// 经度
        longitude: f64,
        title: String,
        content: String,
    },
    Reply {
        message_id: String,
        user_id: String,
    },
}
