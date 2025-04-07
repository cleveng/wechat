use std::collections::HashMap;

use crate::{OfficialAccount, constants::keys};

use deadpool_redis::redis::cmd;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::{Url, form_urlencoded};

#[cfg(test)]
mod tests {
    use crate::OfficialAccount;
    use std::env;

    #[tokio::test]
    async fn get_access_token() {
        dotenv::dotenv().ok();

        let appid = env::var("APPID").expect("APPID not set");
        let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");
        let cfg = env::var("REDIS_URL").expect("APP_CFG not set");

        let account = OfficialAccount::new(appid, app_secret, cfg);
        let at = account.get_qr_ticket("123".to_string()).await;

        println!("get_access_token: {:#?}", at.unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TicketResponse {
    pub ticket: String,
    pub expire_seconds: i64,
    pub url: String,
}

impl OfficialAccount {
    /// [Retrieves the access token for the official account](https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/Get_access_token.html)
    ///
    /// This function attempts to fetch the access token from the Redis database.
    /// If not found, it requests a new token from the WeChat API using the
    /// application ID and secret, then stores the token in Redis for future use.
    ///
    /// # Returns
    ///
    /// * A `Result` containing the access token as a `String` on success, or a boxed
    ///   error on failure.
    ///
    /// # Errors
    ///
    /// * Returns an error if the Redis operation fails, if the HTTP request fails,
    ///   or if the response cannot be deserialized into a `TokenResponse`.
    async fn token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut rdb = self.rdb_pool.get().await?;

        if let Ok(token) = cmd("GET")
            .arg(keys::GLOBAL_TOKEN)
            .query_async::<String>(&mut rdb)
            .await
        {
            return Ok(token);
        }

        let mut url = Url::parse("https://api.weixin.qq.com/cgi-bin/token").unwrap();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("appid", &self.appid)
            .append_pair("secret", &self.app_secret)
            .append_pair("grant_type", "client_credential")
            .finish();

        url.set_query(Some(&query));
        let url = url.to_string();

        let client = Client::new();
        let response = client.get(&url).send().await?;

        let at: TokenResponse = response.json::<TokenResponse>().await?;
        cmd("SETEX")
            .arg(keys::GLOBAL_TOKEN)
            .arg(60 * 50 * 2)
            .arg(serde_json::to_string(&at).unwrap())
            .query_async::<()>(&mut rdb)
            .await
            .unwrap();

        Ok(at.access_token)
    }

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
        scene: String,
    ) -> Result<TicketResponse, Box<dyn std::error::Error>> {
        let token = self.token().await?;

        println!("token: {}", token);

        let mut params = HashMap::new();
        params.insert("expire_seconds".to_string(), "604800".to_string());
        params.insert("action_name".to_string(), "QR_STR_SCENE".to_string());
        params.insert(
            "action_info".to_string(),
            serde_json::json!({ "scene": { "scene_str": scene } }).to_string(),
        );

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/qrcode/create?access_token={}",
            token
        );

        let client = Client::new();
        let response = client.post(url).json(&params).send().await?;
        let result = response.json::<TicketResponse>().await?;

        Ok(result)
    }
}
