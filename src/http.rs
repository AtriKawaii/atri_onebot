use actix_web::{post, web, Responder};

use atri_plugin::bot::Bot;

use crate::data::action::{ActionData, ActionResponse, ActionStatus, BotSelfData};
use crate::data::event::{BotStatus, OneBotMetaStatus};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub access_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct BotQuery {
    bot_id: Option<i64>,
}

#[post("/onebot/get_status")]
pub async fn get_status() -> impl Responder {
    let status = ActionResponse {
        status: ActionStatus::Ok,
        retcode: 0,
        data: Some(ActionData::GetStatus(OneBotMetaStatus {
            good: false,
            bots: Bot::list().into_iter().map(BotStatus::from).collect(),
        })),
        message: "".to_string(),
        echo: None,
    };

    web::Json(status)
}

#[post("/onebot/get_self_info")]
pub async fn get_self_info(bot: web::Query<BotQuery>) -> impl Responder {
    let info = BotSelfData {
        platform: "qq".to_string(),
        user_id: bot.bot_id?.to_string(),
    };

    Some(web::Json(info))
}
