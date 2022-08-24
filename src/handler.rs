use crate::data::action::{
    Action, ActionData, ActionRequest, ActionResponse, ActionStatus, BotSelfData,
    OneBotMessageAction,
};
use crate::data::contact::{GroupInfo, GroupMemberInfo, UserInfo};
use crate::data::event::{BotStatus, OneBotMetaStatus};
use atri_plugin::bot::Bot;
use std::str::FromStr;

macro_rules! id_parse {
    ($id:expr,$echo:ident) => {
        match i64::from_str($id) {
            Ok(id) => id,
            Err(e) => {
                return ActionResponse {
                    status: ActionStatus::Failed,
                    retcode: 10003,
                    data: None,
                    message: e.to_string(),
                    echo: $echo,
                }
            }
        }
    };
}

macro_rules! get_group {
    ($bot:expr, $id:expr, $echo:ident) => {
        if let Some(g) = $bot.find_group($id) {
            g
        } else {
            return ActionResponse {
                status: ActionStatus::Failed,
                retcode: 35002,
                data: None,
                message: "好友不存在".to_string(),
                echo: $echo,
            };
        }
    };
}

macro_rules! get_friend {
    ($bot:expr, $id:expr, $echo:ident) => {
        if let Some(g) = $bot.find_friend($id) {
            g
        } else {
            return ActionResponse {
                status: ActionStatus::Failed,
                retcode: 35003,
                data: None,
                message: "群不存在".to_string(),
                echo: $echo,
            };
        }
    };
}

pub async fn handle_action(req: ActionRequest, bot_id: Option<i64>) -> ActionResponse {
    let ActionRequest {
        action,
        echo,
        bot_self,
    } = req;

    match &action {
        Action::GetStatus {} => {
            return ActionResponse::from_data(
                Some(ActionData::GetStatus(OneBotMetaStatus {
                    good: true,
                    bots: Bot::list().into_iter().map(BotStatus::from).collect(),
                })),
                echo,
            );
        }
        Action::GetSupportActions {} => {
            return ActionResponse::from_data(Some(ActionData::support_actions()), echo);
        }
        Action::GetVersion {} => {
            return ActionResponse::from_data(Some(ActionData::version()), echo);
        }
        _ => {}
    }

    let _echo = echo.clone();
    let get_bot_id = |data: Option<BotSelfData>| -> Result<i64, ActionResponse> {
        if let Some(dat) = data {
            id_parse(&dat.user_id, _echo)
        } else {
            Err(ActionResponse {
                status: ActionStatus::Failed,
                retcode: 10101,
                data: None,
                message: "未指定机器人账号".to_string(),
                echo: _echo,
            })
        }
    };

    let bot_id = if let Some(b) = bot_id {
        b
    } else {
        match get_bot_id(bot_self) {
            Ok(id) => id,
            Err(e) => return e,
        }
    };

    let bot = if let Some(bot) = Bot::find(bot_id) {
        bot
    } else {
        return ActionResponse {
            status: ActionStatus::Failed,
            retcode: 35001,
            data: None,
            message: "机器人不存在或未登陆".to_string(),
            echo,
        };
    };

    let data = match action {
        Action::GetSelfInfo {} => Some(ActionData::GetSelfInfo {
            user_id: bot_id.to_string(),
            user_name: bot.nickname().to_string(),
            user_displayname: "".to_string(),
        }),
        Action::GetUserInfo { user_id } => {
            let id = id_parse!(&user_id, echo);
            let friend = get_friend!(bot, id, echo);

            Some(ActionData::GetUserInfo(UserInfo {
                user_id,
                user_name: friend.nickname().to_string(),
                user_displayname: "".to_string(),
                user_remark: friend.nickname().to_string(),
            }))
        }
        Action::GetFriendList {} => Some(ActionData::GetFriendList(
            bot.friends().into_iter().map(UserInfo::from).collect(),
        )),

        Action::GetGroupInfo { group_id } => {
            let id = id_parse!(&group_id, echo);
            let group = get_group!(bot, id, echo);

            Some(ActionData::GetGroupInfo(GroupInfo {
                group_id,
                group_name: group.name().to_string(),
            }))
        }
        Action::GetGroupList {} => Some(ActionData::GetGroupList(
            bot.groups().into_iter().map(GroupInfo::from).collect(),
        )),
        Action::GetGroupMemberInfo { group_id, user_id } => {
            let g_id = id_parse!(&group_id, echo);
            let u_id = id_parse!(&user_id, echo);

            let group = get_group!(bot, g_id, echo);

            let member = if let Some(named) = group.get_named_member(u_id).await {
                named
            } else {
                return ActionResponse {
                    status: ActionStatus::Ok,
                    retcode: 35004,
                    data: None,
                    message: "群员不存在".to_string(),
                    echo,
                };
            };

            Some(ActionData::GetGroupMemberInfo(GroupMemberInfo {
                user_id,
                user_name: member.nickname().to_string(),
                user_displayname: member.card_name().to_string(),
            }))
        }
        Action::GetGroupMemberList { group_id } => {
            let id = id_parse!(&group_id, echo);

            let group = get_group!(bot, id, echo);

            Some(ActionData::GetGroupMemberList(
                group
                    .members()
                    .await
                    .into_iter()
                    .map(GroupMemberInfo::from)
                    .collect(),
            ))
        }
        Action::SetGroupName {
            group_id,
            group_name,
        } => {
            let id = id_parse!(&group_id, echo);
            let group = get_group!(bot, id, echo);

            if let Err(e) = group.change_name(group_name).await {
                return ActionResponse::from_err(e, 35012, echo);
            }

            None
        }
        Action::LeaveGroup { group_id } => {
            let id = id_parse!(&group_id, echo);
            let group = get_group!(bot, id, echo);

            if !group.quit().await {
                return ActionResponse {
                    status: ActionStatus::Ok,
                    retcode: 35021,
                    data: None,
                    message: "未退出群, 可能是已经退出".to_string(),
                    echo,
                };
            }

            None
        }
        Action::SendMessage(msg) => {
            match msg {
                OneBotMessageAction::Group { message, group_id } => {
                    let id = id_parse!(&group_id, echo);
                }
                OneBotMessageAction::Private { message, user_id } => {
                    let id = id_parse!(&user_id, echo);
                }
                OneBotMessageAction::Channel {
                    message,
                    guild_id,
                    channel_id,
                } => {
                    return ActionResponse {
                        status: ActionStatus::Failed,
                        retcode: 10002,
                        data: None,
                        message: "暂不支持发送频道消息".to_string(),
                        echo,
                    }
                }
            }

            None
        }
        or => {
            return ActionResponse {
                status: ActionStatus::Failed,
                retcode: 20001,
                data: None,
                message: format!("未知动作: {:?}", or),
                echo,
            };
        }
    };

    ActionResponse {
        status: ActionStatus::Ok,
        retcode: 0,
        data,
        message: "".to_string(),
        echo,
    }
}

fn id_parse(id: &str, echo: Option<String>) -> Result<i64, ActionResponse> {
    i64::from_str(id).map_err(|err| ActionResponse {
        status: ActionStatus::Failed,
        retcode: 10003,
        data: None,
        message: err.to_string(),
        echo,
    })
}
