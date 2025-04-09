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
