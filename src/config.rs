use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AtriOneBotConfig {
    #[serde(rename = "server")]
    pub servers: Vec<OneBotServer>,
    pub heartbeat: HeartbeatConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum OneBotServer {
    Http {
        host: String,
        port: u16,
    },
    #[serde(rename = "http-webhook")]
    HttpWebHook {},
    #[serde(rename = "ws")]
    WebSocket {
        host: String,
        port: u16,
        access_token: Option<String>,
    },
    #[serde(rename = "ws-rev")]
    WebSocketReverse,
}

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub enabled: bool,
    pub interval: i64,
}
