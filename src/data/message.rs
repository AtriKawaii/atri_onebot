use atri_plugin::message::meta::Reply;
use atri_plugin::message::{MessageChain, MessageValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Channel {
        #[serde(flatten)]
        message: OneBotMessage,
        guild_id: String,
        channel_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneBotMessage {
    pub message_id: String,
    pub message: Vec<MessageElement>,
    pub alt_message: String,
}

impl From<MessageChain> for OneBotMessage {
    fn from(chain: MessageChain) -> Self {
        let mut ob = Self {
            message_id: "".to_string(),
            message: vec![],
            alt_message: chain.to_string(),
        };

        if let Some(ref reply) = chain.metadata().reply {
            ob.message.push(MessageElement::from(reply));
        }

        let iter = chain.into_iter();

        ob.message.reserve(iter.len());
        for value in iter {
            ob.message.push(MessageElement::from(value));
        }

        ob
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl From<MessageValue> for MessageElement {
    fn from(val: MessageValue) -> Self {
        match val {
            MessageValue::Text(s) => Self::Text { text: s },
            MessageValue::Image(img) => Self::Image {
                file_id: img.id().to_string(),
            },
            MessageValue::At(at) => Self::Mention {
                user_id: at.target.to_string(),
            },
            MessageValue::AtAll => Self::MentionAll {},
            _ => Self::Text {
                text: "no".to_string(),
            },
        }
    }
}

impl From<Reply> for MessageElement {
    fn from(reply: Reply) -> Self {
        Self::from(&reply)
    }
}

impl From<&Reply> for MessageElement {
    fn from(reply: &Reply) -> Self {
        Self::Reply {
            message_id: reply.reply_seq.to_string(),
            user_id: reply.sender.to_string(),
        }
    }
}
