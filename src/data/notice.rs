use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "detail_type")]
pub enum OneBotNoticeEvent {
    FriendIncrease { user_id: String },
    FriendDecrease { user_id: String },
    PrivateMessageDelete { message_id: String, user_id: String },
}
