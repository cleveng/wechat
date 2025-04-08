use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[cfg(test)]
mod tests {
    use crate::official_account::signature;

    #[tokio::test]
    async fn request_validate() {
        let sha1 = "3baae9808b7f85925be470f303cc7b5d035a1c1c".to_string();
        let sha2 = signature::signature("wechat", "1744100071", "952645420");

        println!("签名: {}", sha1 == sha2);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatQuery {
    timestamp: String,
    nonce: String,
    signature: String,
    echostr: Option<String>, // 验证时才需要
}

// actix_web 引入一个中间件，获取 get timestamp nonce  signature 参数 然后验证 signature 是否有效
/// [消息解密](https://developers.weixin.qq.com/doc/offiaccount/Message_Management/Message_encryption_and_decryption_instructions.html)
pub fn signature(token: &str, timestamp: &str, nonce: &str) -> String {
    let mut params = vec![token.to_string(), timestamp.to_string(), nonce.to_string()];
    params.sort();

    let combined = params.join("");

    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();

    format!("{:x}", result)
}
