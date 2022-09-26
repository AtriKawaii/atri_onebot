use crate::data::action::{ActionRequest, ActionResponse};
use crate::data::event::{OneBotEvent, OneBotMetaEvent, OneBotTypedEvent};
use crate::data::message::OneBotMessageEvent;
use crate::handler::handle_action;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use atri_plugin::event::Event;
use atri_plugin::info;
use atri_plugin::listener::{Listener, ListenerGuard};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

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

    tokio::task::spawn_local(async move {
        let interval = 5000;
        assert!(interval > 0);

        let mut heartbeat_pkt = OneBotEvent {
            id: uuid::Uuid::new_v4().to_string(),
            time: SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64(),
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
            heartbeat_pkt.time = SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64();
            tokio::time::sleep(Duration::from_millis(interval as u64)).await; // >0
        }
    });

    let mut event_handler = session.clone();
    tokio::task::spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            let str = serde_json::to_string(&*event);
            match str {
                Ok(str) => {
                    let result = event_handler.text(str).await;
                    if let Err(_) = result {
                        info!(
                            "Websocket 已关闭: {:?}",
                            req.connection_info().realip_remote_addr()
                        );
                        return;
                    }
                }
                Err(e) => {
                    info!("Error: {}", e);
                }
            }
        }
    });

    tokio::task::spawn_local(async move {
        while let Some(Ok(msg)) = stream.recv().await {
            match msg {
                Message::Text(json) => {
                    let rsp = match serde_json::from_str::<ActionRequest>(&json) {
                        Ok(req) => handle_action(req, None).await,
                        Err(e) => ActionResponse::from_err(e, 10001, None),
                    };

                    let str = serde_json::to_string(&rsp).expect("无法序列化OneBot动作响应");
                    if session.text(str).await.is_err() {
                        break;
                    }
                }
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

pub fn ws_listener(tx: tokio::sync::broadcast::Sender<Arc<OneBotEvent>>) -> ListenerGuard {
    Listener::listening_on_always(move |e: Event| {
        let tx = tx.clone();
        async move {
            match e {
                Event::GroupMessageEvent(e) => {
                    let msg = e.message();
                    let ob = OneBotEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        time: msg.metadata().time as f64,
                        inner: OneBotTypedEvent::Message(OneBotMessageEvent::Group {
                            message: msg.into(),
                            group_id: e.group().id().to_string(),
                        }),
                        sub_type: "".to_string(),
                        bot_self: Some(e.bot().into()),
                    };

                    let arc = Arc::new(ob);

                    let _ = tx.send(arc);
                }
                Event::FriendMessageEvent(e) => {
                    let msg = e.message();
                    let ob = OneBotEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        time: msg.metadata().time as f64,
                        inner: OneBotTypedEvent::Message(OneBotMessageEvent::Private {
                            message: msg.into(),
                            user_id: e.friend().id().to_string(),
                        }),
                        sub_type: "".to_string(),
                        bot_self: Some(e.bot().into()),
                    };

                    let arc = Arc::new(ob);

                    let _ = tx.send(arc);
                }
                _ => {}
            }
        }
    })
}
