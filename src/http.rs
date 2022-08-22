use actix_web::{post, web, Responder};

use crate::data::BotSelfData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub access_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct BotList(Vec<i64>);

#[post("/get_bot_list")]
pub async fn get_bot_list() -> impl Responder {
    web::Json(BotList(vec![1, 21, 3]))
}

#[derive(Serialize, Deserialize)]
pub struct BotQuery {
    bot_id: Option<i64>,
}

#[post("/onebot/get_self_info")]
pub async fn get_self_info(bot: web::Query<BotQuery>) -> impl Responder {
    let info = BotSelfData {
        platform: "qq".to_string(),
        user_id: bot.bot_id.unwrap_or(800000).to_string(),
    };

    web::Json(info)
}
