use crate::data::action::{
    Action, ActionData, ActionRequest, ActionResponse, ActionStatus, BotSelfData,
};
use crate::data::event::{BotStatus, OneBotMetaStatus};
use atri_plugin::bot::Bot;
use std::str::FromStr;

pub async fn handle_action(req: ActionRequest, bot_id: Option<i64>) -> ActionResponse {
    let ActionRequest {
        action,
        echo,
        bot_self,
    } = req;

    match &action {
        Action::GetStatus {} => {
            return ActionResponse {
                status: ActionStatus::Ok,
                retcode: 0,
                data: Some(ActionData::GetStatus(OneBotMetaStatus {
                    good: true,
                    bots: Bot::list().into_iter().map(BotStatus::from).collect(),
                })),
                message: "".to_string(),
                echo,
            }
        }
        Action::GetSupportActions {} => {
            return ActionResponse {
                status: ActionStatus::Ok,
                retcode: 0,
                data: Some(ActionData::support_actions()),
                message: "".to_string(),
                echo,
            }
        }
        Action::GetVersion {} => {
            return ActionResponse {
                status: ActionStatus::Ok,
                retcode: 0,
                data: Some(ActionData::version()),
                message: "".to_string(),
                echo,
            }
        }
        _ => {}
    }

    let _echo = echo.clone();
    let get_bot_id = |data: Option<BotSelfData>| -> Result<i64, ActionResponse> {
        if let Some(dat) = data {
            let result = i64::from_str(&dat.user_id);

            return result.map_err(|err| ActionResponse {
                status: ActionStatus::Failed,
                retcode: 10003,
                data: None,
                message: err.to_string(),
                echo: _echo,
            });
        }

        Err(ActionResponse {
            status: ActionStatus::Failed,
            retcode: 10101,
            data: None,
            message: "未指定机器人账号".to_string(),
            echo: _echo,
        })
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

    let (code, data) = match action {
        Action::GetSelfInfo {} => (
            0,
            Some(ActionData::GetSelfInfo {
                user_id: bot_id.to_string(),
                user_name: bot.nickname().to_string(),
                user_displayname: "".to_string(),
            }),
        ),
        _ => {
            return ActionResponse {
                status: ActionStatus::Failed,
                retcode: 20001,
                data: None,
                message: "未知分支, 不应该".to_string(),
                echo,
            };
        }
    };

    let rsp = ActionResponse {
        status: if code == 0 {
            ActionStatus::Ok
        } else {
            ActionStatus::Failed
        },
        retcode: code,
        data,
        message: "".to_string(),
        echo,
    };

    rsp
}
