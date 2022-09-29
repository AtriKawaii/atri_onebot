use crate::data::contact::{GroupInfo, GroupMemberInfo, UserInfo};
use crate::data::event::OneBotMetaStatus;
use crate::data::message::MessageElement;
use atri_plugin::bot::Bot;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize)]
pub struct ActionResponse {
    pub status: ActionStatus,
    pub retcode: i64,
    pub data: Option<ActionData>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ActionData {
    GetSupportActions(&'static [&'static str]),
    GetStatus(OneBotMetaStatus),
    GetVersion {
        #[serde(rename = "impl")]
        implement: &'static str,
        version: &'static str,
        onebot_version: &'static str,
    },
    GetSelfInfo {
        user_id: String,
        user_name: String,
        user_displayname: String,
    },
    GetUserInfo(UserInfo),
    GetFriendList(Vec<UserInfo>),
    GetGroupInfo(GroupInfo),
    GetGroupList(Vec<GroupInfo>),
    GetGroupMemberInfo(GroupMemberInfo),
    GetGroupMemberList(Vec<GroupMemberInfo>),
}

impl ActionData {
    pub fn support_actions() -> Self {
        Self::GetSupportActions(&["get_supported_actions", "get_status", "get_version"])
    }

    pub fn version() -> Self {
        Self::GetVersion {
            implement: "atri-http",
            version: "0.1.0",
            onebot_version: "12",
        }
    }
}

impl ActionResponse {
    pub fn from_err<E: Error>(err: E, code: i64, echo: Option<String>) -> Self {
        Self {
            status: ActionStatus::Failed,
            retcode: code,
            data: None,
            message: err.to_string(),
            echo,
        }
    }

    pub fn from_data(data: Option<ActionData>, echo: Option<String>) -> Self {
        Self {
            status: ActionStatus::Ok,
            retcode: 0,
            data,
            message: "".into(),
            echo,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub bot_self: Option<BotData>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "action", content = "params")]
pub enum Action {
    GetLatestEvents {
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
    LeaveGroup {
        group_id: String,
    },
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
pub struct BotData {
    pub platform: Platform,
    pub user_id: String,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    #[default]
    #[serde(rename = "qq")]
    QQ,
}

impl From<Bot> for BotData {
    fn from(bot: Bot) -> Self {
        Self {
            platform: Platform::QQ,
            user_id: bot.id().to_string(),
        }
    }
}
