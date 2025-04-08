use crate::OfficialAccount;

use serde::{Deserialize, Serialize};
use urlencoding::encode;

const QR_CREATE_URL: &str = "https://api.weixin.qq.com/cgi-bin/qrcode/create?access_token=";
const QR_IMG_URL: &str = "https://mp.weixin.qq.com/cgi-bin/showqrcode?ticket=";

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        OfficialAccount,
        official_account::qrcode::{self, TicketRequest},
    };
    use std::env;

    #[tokio::test]
    async fn get_qr_ticket() {
        dotenv::dotenv().ok();

        let appid = env::var("APPID").expect("appid not set");
        let app_secret = env::var("APP_SECRET").expect("app secret not set");
        let cfg = env::var("REDIS_URL").expect("redis url not set");

        let account = OfficialAccount::new(appid, app_secret, cfg);

        let key = format!("login:{}", Uuid::new_v4().to_string());

        let scene = qrcode::TicketScene {
            scene_id: None,
            scene_str: Some(key.clone()),
        };
        let params = TicketRequest {
            expire_seconds: 200000,
            action_name: qrcode::TicketActionName::QR_STR_SCENE,
            action_info: qrcode::TicketActionInfo { scene },
        };

        let at = account.get_qr_ticket(params).await;
        println!("get_qr_ticket: {:#?}", at.unwrap().show_qrcode());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TicketActionName {
    QR_SCENE,
    QR_STR_SCENE,
    QR_LIMIT_SCENE,
    QR_LIMIT_STR_SCENE,
}

impl TicketActionName {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketActionName::QR_SCENE => "QR_SCENE",
            TicketActionName::QR_STR_SCENE => "QR_STR_SCENE",
            TicketActionName::QR_LIMIT_SCENE => "QR_LIMIT_SCENE",
            TicketActionName::QR_LIMIT_STR_SCENE => "QR_LIMIT_STR_SCENE",
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketActionInfo {
    pub scene: TicketScene,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketScene {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_str: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketRequest {
    expire_seconds: u64,
    action_name: TicketActionName,
    action_info: TicketActionInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketResponse {
    pub ticket: String,
    pub expire_seconds: u64,
    pub url: String,
}

impl TicketResponse {
    /// Generates a URL to display the QR code using the ticket.
    ///
    /// This function encodes the ticket and constructs a URL that can be used to
    /// view the QR code associated with the provided ticket.
    ///
    /// # Returns
    ///
    /// * A `String` representing the URL to display the QR code.
    pub fn show_qrcode(&self) -> String {
        let ticket = encode(&self.ticket);
        format!("{}{}", QR_IMG_URL, ticket)
    }
}

impl OfficialAccount {
    /// [Create a QR code ticket for the official account](https://developers.weixin.qq.com/doc/offiaccount/Account_Management/Generating_a_Parametric_QR_Code.html)
    ///
    /// This function creates a QR code ticket for the official account and returns
    /// a string representing the ticket.
    ///
    /// # Errors
    ///
    /// * Returns an error if the HTTP request fails or returns a non-success status,
    ///   or if the response cannot be deserialized into a `String`.
    pub async fn get_qr_ticket(
        &self,
        req: TicketRequest,
    ) -> Result<TicketResponse, Box<dyn std::error::Error>> {
        let token = self.token().await?;

        let url = format!("{}{}", QR_CREATE_URL, token);
        let response = match self.client.post(url).json(&req).send().await {
            Ok(r) => r,
            Err(e) => {
                println!("json err: {:#?}", e);
                return Err(e.into());
            }
        };

        let result = match response.json::<TicketResponse>().await {
            Ok(r) => r,
            Err(e) => {
                println!("json err: {:#?}", e);
                return Err(e.into());
            }
        };

        Ok(result)
    }
}
