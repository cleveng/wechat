use std::{any::TypeId, str::FromStr};

use crate::OfficialAccount;

use serde::{Deserialize, Serialize};
use urlencoding::encode;

pub(crate) const QR_CREATE_URL: &str =
    "https://api.weixin.qq.com/cgi-bin/qrcode/create?access_token=";
pub(crate) const QR_IMG_URL: &str = "https://mp.weixin.qq.com/cgi-bin/showqrcode?ticket=";

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{
        Config, OfficialAccount,
        official_account::qrcode::{self, TicketRequest},
    };
    use std::env;

    #[tokio::test]
    #[ignore]
    async fn get_qr_ticket() {
        dotenv::dotenv().ok();

        let appid = env::var("APPID").expect("APPID not set");
        let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");
        let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");

        let config = Config {
            appid: appid.clone(),
            app_secret: app_secret.clone(),
            token: "wechat".to_string(),
            encoding_aes_key: None,
        };
        let account = OfficialAccount::new(config, redis_url);

        let key = format!("login:{}", Uuid::new_v4().to_string());

        let scene = qrcode::TicketScene {
            scene_id: None,
            scene_str: Some(key.clone()),
        };
        let params = TicketRequest {
            expire_seconds: 200000,
            action_name: qrcode::TicketActionName::QrStrScene,
            action_info: qrcode::TicketActionInfo { scene },
        };

        let at = account.get_qr_ticket(params).await;
        println!("get_qr_ticket: {:#?}", at.unwrap().show_qrcode());
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TicketActionName {
    QrScene, // => QR_SCENE
    QrStrScene,
    QrLimitScene,
    QrLimitStrScene,
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
    pub expire_seconds: u64,
    pub action_name: TicketActionName,
    pub action_info: TicketActionInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketResponse {
    pub ticket: String,
    pub expire_seconds: u64,
    pub url: String,
}

pub fn build_tmp_qr_request<T>(exp: u64, scene: T) -> TicketRequest
where
    T: 'static + ToString + FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let type_id = TypeId::of::<T>();
    let mut ticket_scene = TicketScene {
        scene_id: None,
        scene_str: None,
    };

    let action_name;

    if type_id == TypeId::of::<String>() || type_id == TypeId::of::<&str>() {
        action_name = TicketActionName::QrStrScene;
        ticket_scene.scene_str = Some(scene.to_string());
    } else {
        action_name = TicketActionName::QrScene;
        let parsed_id = scene
            .to_string()
            .parse::<u32>()
            .expect("scene parse failed");
        ticket_scene.scene_id = Some(parsed_id);
    }

    let expire_seconds = exp.clamp(60, 2592000);

    TicketRequest {
        expire_seconds,
        action_name,
        action_info: TicketActionInfo {
            scene: ticket_scene,
        },
    }
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
        println!("req: {:#?}", &req);
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
