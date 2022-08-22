#![feature(once_cell)]

mod data;
mod http;

use actix_web::dev::{ServerHandle, Service};
use actix_web::{App, HttpResponse, HttpServer, web};
use atri_plugin::{info, Plugin};
use std::future::Future;
use std::mem;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use crate::http::{get_bot_list, get_self_info};
use actix_web::http::header::Header;
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use atri_plugin::runtime::manager::PluginManager;

#[atri_plugin::plugin]
struct AtriOneBot {
    server: Option<WebServer>,
}

struct WebServer {
    runtime: tokio::runtime::Runtime,
    handle: ServerHandle,
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

        let http_server = HttpServer::new(|| {
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
                .service(get_bot_list)
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

        self.server = Some(WebServer {
            runtime: rt,
            handle
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
    use crate::data::event::{OneBotEvent, OneBotEventInner, OneBotSelfEvent};
    use crate::data::message::{MessageElement, OneBotMessage, OneBotMessageEvent};
    use crate::data::BotSelfData;
    use crate::http::{get_bot_list, get_self_info};
    use actix_web::dev::ServerHandle;
    use actix_web::{get, web, App, HttpServer, Responder};
    use serde::{Deserialize, Serialize};
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
                    .service(get_bot_list)
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
        #[derive(Serialize, Deserialize)]
        struct A(f32);

        #[derive(Serialize, Deserialize)]
        struct B {
            a: A,
            c: C,
        }

        #[derive(Serialize, Deserialize)]
        #[serde(tag = "type")]
        enum C {
            B(D),
        }

        #[derive(Serialize, Deserialize)]
        #[serde(tag = "sub_type")]
        enum D {
            A,
        }

        let data = OneBotSelfEvent {
            inner: OneBotEvent {
                id: "".to_string(),
                time: 0.0,
                inner: OneBotEventInner::Message(OneBotMessageEvent::Private {
                    message: OneBotMessage {
                        message_id: "".to_string(),
                        message: vec![MessageElement::MentionAll {}],
                        alt_message: "".to_string(),
                    },
                    user_id: "1145141919".to_string(),
                }),
                sub_type: "h".to_string(),
            },
            bot_self: BotSelfData {
                platform: "qq".to_string(),
                user_id: "114514".to_string(),
            },
        };

        let _val = serde_json::to_value(&data).unwrap();

        println!("{}", serde_json::to_string_pretty(&data).unwrap());
    }
}
