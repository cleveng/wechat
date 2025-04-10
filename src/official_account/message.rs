use chrono::Utc;
use quick_xml::{de::from_str, se};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

use actix_web::{
    FromRequest, HttpRequest,
    dev::Payload,
    error,
    web::{self},
};

use super::signature::signature;

pub struct MsgType;

impl MsgType {
    // TEXT 表示文本消息
    pub const TEXT: &str = "text";
    // IMAGE 表示图片消息
    pub const IMAGE: &str = "image";
    // VOICE 表示语音消息
    pub const VOICE: &str = "voice";
    // VIDEO 表示视频消息
    pub const VIDEO: &str = "video";
    // MINIPROGRAMPAGE 表示小程序卡片消息
    pub const MINIPROGRAMPAGE: &str = "miniprogrampage";
    // SHORTVIDEO 表示短视频消息 [限接收]
    pub const SHORTVIDEO: &str = "shortvideo";
    // LOCATION 表示坐标消息 [限接收]
    pub const LOCATION: &str = "location";
    // LINK 表示链接消息 [限接收]
    pub const LINK: &str = "link";
    // MUSIC 表示音乐消息 [限回复]
    pub const MUSIC: &str = "music";
    // NEWS 表示图文消息 [限回复]
    pub const NEWS: &str = "news";
    // TRANSFER 表示消息消息转发到客服
    pub const TRANSFER: &str = "transfer_customer_service";
    // EVENT 表示事件推送消息
    pub const EVENT: &str = "event";
}

pub struct EventType;

impl EventType {
    // SUBSCRIBE 订阅
    pub const SUBSCRIBE: &str = "subscribe";
    // UNSUBSCRIBE 取消订阅
    pub const UNSUBSCRIBE: &str = "unsubscribe";
    // SCAN 取消订阅
    pub const SCAN: &str = "SCAN";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "xml")]
pub struct WechatMessage {
    #[serde(rename = "ToUserName")]
    pub to_user_name: String, // 微信号(公众号原始id)
    #[serde(rename = "FromUserName")]
    pub from_user_name: String, // 发送方账号（一个OpenID）
    #[serde(rename = "CreateTime")]
    pub create_time: u64, // 消息创建时间 （整型）
    #[serde(rename = "MsgType")]
    pub msg_type: String, // 消息类型，文本为text
    #[serde(rename = "Content")]
    pub content: Option<String>, // 文本消息内容
    #[serde(rename = "MsgId")]
    pub msg_id: Option<u64>, // 消息id，64位整型
    #[serde(rename = "Idx")]
    pub idx: Option<u64>, // 多图文时第几篇文章，从1开始（消息如果来自文章时才有）
    #[serde(rename = "Event")]
    pub event: Option<String>, // 事件类型，subscribe
    #[serde(rename = "EventKey")]
    pub event_key: Option<String>, // 事件KEY值，qrscene_为前缀，后面为二维码的场景值ID
    #[serde(rename = "Ticket")]
    pub ticket: Option<String>, // 二维码的ticket，可用来换取二维码图片
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "xml")]
pub struct WeChatResponse {
    #[serde(rename = "ToUserName")]
    pub to_user_name: String,
    #[serde(rename = "FromUserName")]
    pub from_user_name: String,
    #[serde(rename = "CreateTime")]
    pub create_time: u64,
    #[serde(rename = "MsgType")]
    pub msg_type: String,
    #[serde(rename = "Content")]
    pub content: String,
}

pub struct MessageHandler {
    pub message: WechatMessage,
}

impl MessageHandler {
    pub fn to_string(
        &self,
        message: &WeChatResponse,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match se::to_string(message) {
            Ok(s) => Ok(s),
            Err(e) => Err(e.into()),
        }
    }
}

impl WechatMessage {
    pub fn get_open_id(&self) -> String {
        self.from_user_name.clone()
    }

    pub fn get_gh_id(&self) -> String {
        self.to_user_name.clone()
    }

    pub fn plaintext(&self, content: &str) -> WeChatResponse {
        let now = Utc::now().naive_utc();
        WeChatResponse {
            to_user_name: self.from_user_name.clone(),
            from_user_name: self.to_user_name.clone(),
            create_time: now.and_utc().timestamp() as u64,
            msg_type: MsgType::TEXT.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatQuery {
    timestamp: String,
    nonce: String,
    signature: String,
    echostr: Option<String>, // 验证时才需要
}

impl FromRequest for MessageHandler {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Bytes::from_request(req, payload);

        let query_future = web::Query::<WechatQuery>::from_query(req.query_string())
            .map(|q| q.into_inner())
            .map_err(|e| {
                actix_web::error::ErrorBadRequest(format!(
                    "Failed to parse query parameters: {}",
                    e
                ))
            });

        Box::pin(async move {
            let query = match query_future {
                Ok(q) => q,
                Err(e) => return Err(e),
            };

            // TODO: token use official account
            if query.signature != signature("wechat", &query.timestamp, &query.nonce) {
                return Err(error::ErrorUnauthorized("Invalid signature"));
            }

            let bytes = fut.await?;
            let xml_str = String::from_utf8_lossy(&bytes);

            let message = from_str::<WechatMessage>(&xml_str)
                .map_err(|e| error::ErrorBadRequest(format!("Invalid XML input: {}", e)))?;

            Ok(MessageHandler { message })
        })
    }
}
