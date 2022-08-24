use actix_web::{post, web, HttpResponse, Responder};

use atri_plugin::bot::Bot;

use crate::data::action::{
    Action, ActionData, ActionRequest, ActionResponse, ActionStatus, BotSelfData,
};
use crate::data::event::{BotStatus, OneBotMetaStatus};
use crate::handler::handle_action;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Authorization {
    pub access_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct BotQuery {
    bot_id: Option<i64>,
}

#[post("/onebot")]
pub async fn onebot_http(req: String) -> impl Responder {
    let rsp = match serde_json::from_str::<ActionRequest>(&req) {
        Ok(req) => handle_action(req, None).await,
        Err(e) => ActionResponse::from_err(e, 10001, None),
    };
    web::Json(rsp)
}
