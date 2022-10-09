use crate::config::HeartbeatConfig;
use crate::data::action::{ActionRequest, ActionResponse};
use crate::data::event::{BotStatus, OneBotEvent, OneBotMetaEvent, OneBotMetaStatus, OneBotTypedEvent};
use crate::data::message::OneBotMessageEvent;
use crate::handler::handle_action;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message;
use atri_plugin::event::Event;
use atri_plugin::info;
use atri_plugin::listener::{Listener, ListenerGuard};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};
use atri_plugin::bot::Bot;

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

    let heartbeat = if let Some(h) = req.app_data::<HeartbeatConfig>() {
        *h
    } else {
        return HttpResponse::ExpectationFailed().await;
    };

    let remote = req
        .connection_info()
        .realip_remote_addr()
        .map(|str: &str| str.to_owned());

    let (resp, mut session, mut stream) = actix_ws::handle(&req, stream)?;

    info!("WebSocket已连接, Remote address: {:?}", remote);
    let mut heartbeat_session = session.clone();

    if heartbeat.enabled {
        tokio::task::spawn_local(async move {
            let interval = heartbeat.interval;
            assert!(interval > 0);

            let mut heartbeat_pkt = OneBotEvent {
                id: uuid::Uuid::new_v4().to_string(),
                time: sys_time(),
                typed: OneBotTypedEvent::Meta(OneBotMetaEvent::Heartbeat { interval }),
                sub_type: "",
                bot_self: None,
            };

            while let Ok(()) = heartbeat_session
                .text(serde_json::to_string(&heartbeat_pkt).unwrap_or_default())
                .await
            {
                let uuid = uuid::Uuid::new_v4();
                heartbeat_pkt.id = uuid.to_string();
                heartbeat_pkt.time = sys_time();
                tokio::time::sleep(Duration::from_millis(interval as u64)).await;
                // >0
            }
        });
    }

    let mut event_handler = session.clone();
    tokio::task::spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            let str = serde_json::to_string(&*event);
            match str {
                Ok(str) => {
                    let result = event_handler.text(str).await;
                    if let Err(_) = result {
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
                        Ok(req) => handle_action(req).await,
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
                Message::Close(reason) => {
                    info!(
                        "WebSocket已关闭, 原因: {:?}, Remote address: {:?}",
                        reason, remote
                    );

                    break;
                }
                Message::Nop => {}
            }
        }
    });

    Ok(resp)
}

pub fn listener(tx: tokio::sync::broadcast::Sender<Arc<OneBotEvent>>) -> ListenerGuard {
    let counter = Arc::new(AtomicBool::new(false));

    let cnt = counter.clone();
    let sender = tx.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            if !cnt.swap(false, Ordering::Acquire) {
                let ob = OneBotEvent {
                    id: uuid::Uuid::new_v4().to_string(),
                    time: sys_time(),
                    typed: OneBotTypedEvent::Meta(OneBotMetaEvent::StatusUpdate {
                        status: OneBotMetaStatus {
                            good: true,
                            bots: Bot::list().into_iter().map(BotStatus::from).collect()
                        }
                    }),
                    sub_type: "",
                    bot_self: None
                };

                let arc = Arc::new(ob);

                let _ = sender.send(arc);

                break;
            }
        }
    });

    Listener::listening_on_always(move |e: Event| {
        let tx = tx.clone();
        let cnt = counter.clone();
        async move {
            match e {
                Event::BotLogin(_) => {
                    cnt.swap(true, Ordering::Relaxed);
                }
                Event::GroupMessage(e) => {
                    let msg = e.message();
                    let ob = OneBotEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        time: msg.metadata().time as f64,
                        typed: OneBotTypedEvent::Message(OneBotMessageEvent::Group {
                            message: msg.into(),
                            group_id: e.group().id().to_string(),
                        }),
                        sub_type: "",
                        bot_self: Some(e.bot().into()),
                    };

                    let arc = Arc::new(ob);

                    let _ = tx.send(arc);
                }
                Event::FriendMessage(e) => {
                    let msg = e.message();
                    let ob = OneBotEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        time: msg.metadata().time as f64,
                        typed: OneBotTypedEvent::Message(OneBotMessageEvent::Private {
                            message: msg.into(),
                            user_id: e.friend().id().to_string(),
                        }),
                        sub_type: "",
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

fn sys_time() -> f64 {
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64()
}