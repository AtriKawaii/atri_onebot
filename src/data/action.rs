use crate::data::event::OneBotMetaStatus;
use crate::data::message::MessageElement;
use atri_plugin::bot::Bot;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct ActionResponse {
    pub status: ActionStatus,
    pub retcode: i64,
    pub data: Option<ActionData>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionData {
    GetSupportActions(Vec<String>),
    GetStatus(OneBotMetaStatus),
    GetVersion {
        #[serde(rename = "impl")]
        implement: String,
        version: String,
        onebot_version: String,
    },
    GetSelfInfo {
        user_id: String,
        user_name: String,
        user_displayname: String,
    },
}

impl ActionData {
    pub fn support_actions() -> Self {
        Self::GetSupportActions(
            ["get_supported_actions", "get_status", "get_version"]
                .into_iter()
                .map(String::from)
                .collect(),
        )
    }

    pub fn version() -> Self {
        Self::GetVersion {
            implement: "atri-http".to_string(),
            version: "0.1.0".to_string(),
            onebot_version: "12".to_string(),
        }
    }
}

impl ActionResponse {
    pub fn from_err<E: Error>(err: E, code: i64) -> Self {
        Self {
            status: ActionStatus::Failed,
            retcode: code,
            data: None,
            message: err.to_string(),
            echo: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    Ok,
    Failed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRequest {
    #[serde(flatten)]
    pub action: Action,
    pub echo: Option<String>,
    #[serde(rename = "self")]
    pub bot_self: Option<BotSelfData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "action", content = "params")]
pub enum Action {
    GetLatestEvent {
        limit: i64,
        timeout: i64,
    },
    GetSupportActions {},
    GetSelfInfo {},
    GetStatus {},
    GetVersion {},
    GetUserInfo {
        user_id: String,
    },
    GetFriendList {},
    GetGroupInfo {
        group_id: String,
    },
    GetGroupList {},
    GetGroupMemberInfo {
        group_id: String,
        user_id: String,
    },
    GetGroupMemberList {
        group_id: String,
    },
    SetGroupName {
        group_id: String,
        group_name: String,
    },
    LeaveGroup {},
    SendMessage(OneBotMessageAction),
    DeleteMessage {
        message_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "detail_type")]
pub enum OneBotMessageAction {
    Private {
        message: Vec<MessageElement>,
        user_id: String,
    },
    Group {
        message: Vec<MessageElement>,
        group_id: String,
    },
    Channel {
        message: Vec<MessageElement>,
        guild_id: String,
        channel_id: String,
    },
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
