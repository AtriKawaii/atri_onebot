use std::fs::{create_dir_all, File, OpenOptions};
use std::future::Future;
use std::io::{Read, Write};
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{AtriOneBotConfig, OneBotServer};
use actix_web::dev::{ServerHandle, Service};
use actix_web::http::header::Header;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use atri_plugin::listener::ListenerGuard;
use atri_plugin::{error, info, Plugin};

use crate::http::onebot_http;
use crate::websocket::{listener, start_websocket};

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
    handles: Vec<ServerHandle>,
    _listener: ListenerGuard,
}

static CONFIG_DIR: &str = "workspaces/atri_onebot";
static CONFIG_FILE: &str = "config.toml";

impl Plugin for AtriOneBot {
    fn new() -> Self {
        Self { server: None }
    }

    fn enable(&mut self) {
        let mut path = PathBuf::from(CONFIG_DIR);
        create_dir_all(&path).expect("Cannot create workspace for atri_onebot");
        path.push(CONFIG_FILE);

        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .expect("Cannot open or create config file");

        let mut bytes = vec![];
        (&f).read_to_end(&mut bytes).expect("Cannot read file");
        drop(f);

        let config: AtriOneBotConfig = toml::from_slice(&bytes).unwrap_or_else(|e| {
            error!("读取配置文件失败: {}", e);

            let c = AtriOneBotConfig::default();
            let str = toml::to_string_pretty(&c).unwrap();
            File::create(&path)
                .expect("Cannot create file")
                .write_all(str.as_bytes())
                .expect("Cannot write config to file");

            c
        });

        println!("Config: {:?}", config);

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let (tx, _) = tokio::sync::broadcast::channel(61);

        let mut handles = vec![];

        let mut heartbeat = config.heartbeat;
        if heartbeat.interval <= 0 {
            heartbeat.enabled = false;
        }
        for server in config.servers {
            match server {
                OneBotServer::WebSocket {
                    host,
                    port,
                    access_token,
                } => {
                    let server_tx = tx.clone();

                    let token = Arc::new(access_token);

                    let http_server = HttpServer::new(move || {
                        let token = Arc::clone(&token);

                        App::new()
                            .wrap_fn(move |a, b| {
                                let correct = &**token;

                                let f: Box<dyn Future<Output = _>> =
                                    if let Ok(auth) = Authorization::<Bearer>::parse(&a) {
                                        let bearer = auth.into_scheme();
                                        if bearer.token() == correct {
                                            let b = b.call(a);
                                            Box::new(b)
                                        } else {
                                            Box::new(async {
                                                Ok(a.into_response(
                                                    HttpResponse::Unauthorized().finish(),
                                                ))
                                            })
                                        }
                                    } else {
                                        let query = a.query_string();
                                        let auth: String = serde_urlencoded::from_str(query)
                                            .map(|e: http::Authorization| e.access_token)
                                            .unwrap_or_else(|_| String::new());

                                        if auth == correct {
                                            let b = b.call(a);
                                            Box::new(b)
                                        } else {
                                            Box::new(async {
                                                Ok(a.into_response(
                                                    HttpResponse::Unauthorized().finish(),
                                                ))
                                            })
                                        }
                                    };

                                async move {
                                    let pin = Box::into_pin(f);
                                    pin.await
                                }
                            })
                            .service(
                                web::resource("/onebot12/websocket")
                                    .route(web::get().to(start_websocket))
                                    .app_data(server_tx.clone())
                                    .app_data(heartbeat),
                            )
                            .service(onebot_http)
                            .default_service(web::to(|| async { "Unknown" }))
                    })
                    .bind((host, port))
                    .unwrap()
                    .workers(4)
                    .run();

                    handles.push(http_server.handle());

                    rt.spawn(async move {
                        http_server.await.unwrap();
                    });
                }
                _ => todo!(),
            }
        }

        let tx = tx.clone();
        let guard = listener(tx);

        self.server = Some(WebServer {
            runtime: rt,
            handles,
            _listener: guard,
        });

        info!("Started");
    }
}

impl Drop for AtriOneBot {
    fn drop(&mut self) {
        if let Some(server) = mem::take(&mut self.server) {
            server.handles.iter().for_each(|handle| {
                let _ = handle.stop(true);
            });

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
            let _ = HttpServer::new(|| {
                App::new()
                    .service(hello)
                    .default_service(web::to(|| async { "Where are u" }))
            })
            .bind(("127.0.0.1", 8080))
            .unwrap()
            .run();
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
