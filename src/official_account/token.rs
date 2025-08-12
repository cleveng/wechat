use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use url::{Url, form_urlencoded};

use crate::OfficialAccount;
use crate::constants::keys;

pub(crate) const TOKEN_URL: &str = "https://api.weixin.qq.com/cgi-bin/token";

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    expires_in: u64,
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
    pub async fn token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut rdb = self.rdb_pool.get().await?;

        let value: Option<String> = cmd("GET")
            .arg(keys::GLOBAL_TOKEN)
            .query_async(&mut rdb)
            .await
            .unwrap_or(None);
        if let Some(bytes) = value {
            let at: TokenResponse = serde_json::from_str(&bytes).unwrap();
            return Ok(at.access_token);
        }

        let mut url = Url::parse(TOKEN_URL).unwrap();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("appid", &self.config.appid)
            .append_pair("secret", &self.config.app_secret)
            .append_pair("grant_type", "client_credential")
            .finish();

        url.set_query(Some(&query));
        let url = url.to_string();

        let response = self.client.get(&url).send().await?;
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
}
