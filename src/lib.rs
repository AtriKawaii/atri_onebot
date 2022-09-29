use std::future::Future;
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use actix_web::dev::{ServerHandle, Service};
use actix_web::http::header::Header;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use atri_plugin::listener::ListenerGuard;
use atri_plugin::{info, Plugin};

use crate::http::onebot_http;
use crate::websocket::{start_websocket, ws_listener};

mod config;
mod data;
mod handler;
mod http;
mod websocket;

#[atri_plugin::plugin]
struct AtriOneBot {
    server: Option<WebServer>,
}

struct WebServer {
    runtime: tokio::runtime::Runtime,
    handle: ServerHandle,
    _listener: ListenerGuard,
}

static CONFIG_PATH: &str = "config/atri_onebot";

impl Plugin for AtriOneBot {
    fn new() -> Self {
        Self { server: None }
    }

    fn enable(&mut self) {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let addr = SocketAddrV4::new(ip, 8080);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
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
                                    Ok(a.into_response(HttpResponse::Unauthorized().finish()))
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
                                    Ok(a.into_response(HttpResponse::Unauthorized().finish()))
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
                .service(onebot_http)
                .default_service(web::to(|| async { "Unknown" }))
        })
        .bind(addr)
        .unwrap()
        .workers(4)
        .run();

        let handle = http_server.handle();

        rt.spawn(async move {
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
    use actix_web::{get, web, App, HttpServer, Responder};
    use serde_json::json;

    use crate::data::action::ActionRequest;

    #[test]
    fn test_server() {
        #[get("/")]
        async fn hello() -> impl Responder {
            "HttpResponseBuilder::new(StatusCode::BAD_REQUEST)"
        }

        actix_web::rt::Runtime::new().unwrap().block_on(async {
            let http_server = HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .default_service(web::to(|| async { "Where are u" }))
            })
            .bind(("127.0.0.1", 8080))
            .unwrap()
            .run();
            http_server.await.unwrap();
        });
    }

    #[test]
    fn actions() {
        let get_latest_events = json!({
            "action": "get_latest_events",
            "params": {
                "limit": 100,
                "timeout": 0
            }
        });

        println!(
            "{:?}",
            serde_json::from_value::<ActionRequest>(get_latest_events).unwrap()
        );

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
