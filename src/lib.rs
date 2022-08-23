#![feature(once_cell)]

mod data;
mod http;
mod websocket;

use actix_web::dev::{ServerHandle, Service};
use actix_web::{web, App, HttpResponse, HttpServer};
use atri_plugin::{info, Plugin};
use std::future::Future;
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};

use std::time::Duration;

use crate::http::{get_self_info, get_status};
use crate::websocket::{start_websocket, ws_listener};
use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use atri_plugin::listener::ListenerGuard;

#[atri_plugin::plugin]
struct AtriOneBot {
    server: Option<WebServer>,
}

struct WebServer {
    runtime: tokio::runtime::Runtime,
    handle: ServerHandle,
    _listener: ListenerGuard,
}

impl Plugin for AtriOneBot {
    fn new() -> Self {
        Self { server: None }
    }

    fn enable(&mut self) {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let addr = SocketAddrV4::new(ip, 8080);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, _) = tokio::sync::broadcast::channel(61);

        let server_tx = tx.clone();

        let http_server = HttpServer::new(move || {
            App::new()
                .wrap_fn(|a, b| {
                    let correct = String::from("1234");

                    let f: Box<dyn Future<Output = _>> =
                        if let Ok(auth) = Authorization::<Bearer>::parse(&a) {
                            let bearer = auth.into_scheme();
                            if bearer.token() == correct {
                                let b = b.call(a);
                                Box::new(async { b.await })
                            } else {
                                Box::new(async {
                                    Ok(a.into_response(HttpResponse::Unauthorized().await?))
                                })
                            }
                        } else {
                            let query = a.query_string();
                            let auth: String = serde_urlencoded::from_str(query)
                                .map(|e: http::Authorization| e.access_token)
                                .unwrap_or_else(|_| String::new());

                            if auth == correct {
                                let b = b.call(a);
                                Box::new(async { b.await })
                            } else {
                                Box::new(async {
                                    Ok(a.into_response(HttpResponse::Unauthorized().await?))
                                })
                            }
                        };

                    async move {
                        let pin = Box::into_pin(f);
                        pin.await
                    }
                })
                .service(
                    web::resource("/onebot/websocket")
                        .route(web::get().to(start_websocket))
                        .app_data(server_tx.clone()),
                )
                .service(get_status)
                .service(get_self_info)
                .default_service(web::to(|| async { "Unknown" }))
        })
        .bind(addr)
        .unwrap()
        .workers(4)
        .run();

        let handle = http_server.handle();

        rt.spawn(async {
            http_server.await.unwrap();
        });

        let tx = tx.clone();
        let guard = ws_listener(tx);

        self.server = Some(WebServer {
            runtime: rt,
            handle,
            _listener: guard,
        });

        info!("Started");
    }

    fn should_drop() -> bool {
        true
    }
}

impl Drop for AtriOneBot {
    fn drop(&mut self) {
        if let Some(server) = mem::take(&mut self.server) {
            let _ = server.handle.stop(true);
            server.runtime.shutdown_timeout(Duration::from_millis(800));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::data::action::{ActionRequest, BotSelfData};
    use crate::data::event::{
        BotStatus, OneBotEvent, OneBotMetaEvent, OneBotMetaStatus, OneBotTypedEvent,
    };
    use crate::http::get_self_info;
    use actix_web::dev::ServerHandle;
    use actix_web::{get, web, App, HttpServer, Responder};
    use serde_json::json;
    use std::sync::OnceLock;

    #[test]
    fn test_server() {
        #[get("/")]
        async fn hello() -> impl Responder {
            "HttpResponseBuilder::new(StatusCode::BAD_REQUEST)"
        }

        #[get("/stop")]
        async fn stop() -> impl Responder {
            let _ = HANDLE.get().unwrap().stop(true);
            ""
        }

        static HANDLE: OnceLock<ServerHandle> = OnceLock::new();

        actix_web::rt::Runtime::new().unwrap().block_on(async {
            let http_server = HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .service(stop)
                    .service(get_self_info)
                    .default_service(web::to(|| async { "Where are u" }))
            })
            .bind(("127.0.0.1", 8080))
            .unwrap()
            .run();
            HANDLE.get_or_init(|| http_server.handle());
            http_server.await.unwrap();
        });
    }

    #[test]
    fn json() {
        let data = OneBotEvent {
            id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_string(),
            time: 1632847927.599013,
            inner: OneBotTypedEvent::Meta(OneBotMetaEvent::StatusUpdate {
                status: OneBotMetaStatus {
                    good: true,
                    bots: vec![
                        BotStatus {
                            bot_self: BotSelfData {
                                platform: "qq".to_string(),
                                user_id: "123456".to_string(),
                            },
                            online: true,
                            ext: None,
                        },
                        BotStatus {
                            bot_self: BotSelfData {
                                platform: "qq".to_string(),
                                user_id: "114514".to_string(),
                            },
                            online: true,
                            ext: None,
                        },
                    ],
                },
            }),
            sub_type: "".to_string(),
            bot_self: None,
        };

        let str = serde_json::to_string_pretty(&data).unwrap();

        println!("{}", str);

        let _e: OneBotEvent = serde_json::from_str(&str).unwrap();
    }

    #[test]
    fn actions() {
        let self_info_req = json!({
            "action": "get_self_info",
            "params": {}
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(self_info_req).unwrap()
        );

        let user_info_req = json!({
            "action": "get_user_info",
            "params": {
                "user_id": "114514"
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(user_info_req).unwrap()
        );

        let friend_list_req = json!({
            "action": "get_friend_list",
            "params": {}
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(friend_list_req).unwrap()
        );

        let send_message_req = json!({
            "action": "send_message",
            "params": {
                "detail_type": "group",
                "group_id": "12467",
                "message": [
                    {
                        "type": "text",
                        "data": {
                            "text": "我是文字巴拉巴拉巴拉"
                        }
                    }
                ]
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(send_message_req).unwrap()
        );

        let group_info_req = json!({
            "action": "get_group_info",
            "params": {
                "group_id": "123456"
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(group_info_req).unwrap()
        );

        let group_list_req = json!({
            "action": "get_group_list",
            "params": {}
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(group_list_req).unwrap()
        );

        let group_member_info_req = json!({
                "action": "get_group_member_info",
                "params": {
                "group_id": "123456",
                "user_id": "3847573"
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(group_member_info_req).unwrap()
        );

        let group_member_list_req = json!({
                "action": "get_group_member_list",
                "params": {
                "group_id": "114514",
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(group_member_list_req).unwrap()
        );

        let set_group_name_req = json!({
                "action": "set_group_name",
                "params": {
                "group_id": "123456",
                "group_name": "1919810"
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(set_group_name_req).unwrap()
        );
    }
}
