use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use actix_ws::Message;
use std::sync::Arc;
use std::time::{Duration, Instant};

use atri_plugin::bot::Bot;
use atri_plugin::info;

use crate::data::event::{
    BotStatus, OneBotEvent, OneBotMetaEvent, OneBotMetaStatus, OneBotTypedEvent,
};
use crate::data::{ActionData, ActionResponse, ActionStatus, BotSelfData};
use serde::{Deserialize, Serialize};

pub async fn start_websocket(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let mut rx = if let Some(t) = req.app_data::<tokio::sync::broadcast::Sender<Arc<OneBotEvent>>>()
    {
        t.subscribe()
    } else {
        return HttpResponse::ExpectationFailed().await;
    };

    let (resp, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    info!(
        "Websocket已连接: {:?}",
        req.connection_info().realip_remote_addr()
    );
    let mut heartbeat = session.clone();

    actix_web::rt::spawn(async move {
        let interval = 5000;

        let mut heartbeat_pkt = OneBotEvent {
            id: uuid::Uuid::new_v4().to_string(),
            time: Instant::now().elapsed().as_secs_f64(),
            inner: OneBotTypedEvent::Meta(OneBotMetaEvent::Heartbeat { interval }),
            sub_type: "".to_string(),
            bot_self: None,
        };

        while let Ok(()) = heartbeat
            .text(serde_json::to_string(&heartbeat_pkt).unwrap_or_default())
            .await
        {
            let uuid = uuid::Uuid::new_v4();
            heartbeat_pkt.id = uuid.to_string();
            tokio::time::sleep(Duration::from_millis(interval as u64)).await; // >0
        }
    });

    actix_web::rt::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let str = serde_json::to_string(&*event);
            match str {
                Ok(str) => {
                    let result = session.text(str).await;
                    if result.is_err() {
                        return;
                    }
                }
                Err(e) => {
                    info!("Error: {}", e);
                }
            }
        }
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = stream.recv().await {
            match msg {
                Message::Text(s) => {}
                Message::Binary(_) => {}
                Message::Continuation(_) => {}
                Message::Ping(_) => {}
                Message::Pong(_) => {}
                Message::Close(_) => {
                    break;
                }
                Message::Nop => {}
            }
        }
    });

    Ok(resp)
}

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
