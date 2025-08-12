use crate::OfficialAccount;

use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use url::{Url, form_urlencoded};

pub(crate) const OAUTH2_URL: &str = "https://open.weixin.qq.com/connect/oauth2/authorize";

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{Config, OfficialAccount};

    #[test]
    fn get_redirect_url() {
        dotenv::dotenv().ok();

        let appid = env::var("APPID").expect("APPID not set");
        let app_secret = env::var("APP_SECRET").expect("APP_SECRET not set");
        let redis_url = env::var("REDIS_URL").expect("REDIS_URL not set");
        let redirect_uri = env::var("REDIRECT_URI").expect("REDIRECT_URI not set");

        let config = Config {
            appid: appid.clone(),
            app_secret: app_secret.clone(),
            token: "wechat".to_string(),
            encoding_aes_key: None,
        };
        let account = OfficialAccount::new(config, redis_url);

        let url = account.get_redirect_url(redirect_uri, "snsapi_userinfo".to_string(), None);
        println!("url: {:#?}", url);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    #[serde(rename = "openid")]
    pub open_id: String,
    scope: String,
    #[serde(rename = "unionid")]
    pub union_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BasicResponse {
    pub errcode: i64,
    pub errmsg: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoResponse {
    #[serde(rename = "openid")]
    pub open_id: String,
    pub nickname: Option<String>,
    pub sex: Option<i64>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "headimgurl")]
    pub profile_url: Option<String>,
    pub privilege: Option<Vec<String>>,
    #[serde(rename = "unionid")]
    pub union_id: Option<String>,
}

impl OfficialAccount {
    /// [获取跳转的url地址](https://developers.weixin.qq.com/doc/offiaccount/OA_Web_Apps/Wechat_webpage_authorization.html)
    pub fn get_redirect_url(
        &self,
        redirect_uri: String,
        scope: String,
        state: Option<String>,
    ) -> String {
        let mut url = Url::parse(OAUTH2_URL).unwrap();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("appid", &self.config.appid)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &scope)
            .append_pair("state", state.as_deref().unwrap_or(""))
            .finish();

        url.set_query(Some(&query));
        format!("{}#wechat_redirect", url.to_string())
    }

    /// [Exchanges the given authorization code for an access token using the WeChat API](https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/get_oauth2_token.html)
    ///
    /// This function constructs a request to the WeChat OAuth2 API endpoint with the
    /// provided authorization code, app ID, and app secret, and processes the response
    /// to retrieve an access token.
    ///
    /// # Arguments
    ///
    /// * `code` - A `String` containing the authorization code received from the WeChat
    ///   authorization server.
    ///
    /// # Returns
    ///
    /// * A `Result` containing an `AccessTokenResponse` on success, or a boxed error on failure.
    ///
    /// # Errors
    ///
    /// * Returns an error if the HTTP request fails or returns a non-success status, or if
    ///   the response cannot be deserialized into an `AccessTokenResponse`.
    pub async fn get_oauth2_token(
        &self,
        code: String,
    ) -> Result<AccessTokenResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code",
            self.config.appid, self.config.app_secret, code
        );

        println!("url: {}", url);

        let response = self.client.get(&url).send().await?;
        let at: AccessTokenResponse = response.json::<AccessTokenResponse>().await?;

        let mut rdb = self.rdb_pool.get().await.unwrap();
        cmd("SETEX")
            .arg(&at.open_id)
            .arg(24 * 60)
            .arg(serde_json::to_string(&at).unwrap())
            .query_async::<()>(&mut rdb)
            .await
            .unwrap();

        Ok(at)
    }

    /// Retrieves user information from WeChat API using the provided `openid`.
    ///
    /// This function fetches the access token from the Redis database associated
    /// with the given `openid`, constructs a request to the WeChat API to get
    /// user information, and processes the response.
    ///
    /// # Arguments
    ///
    /// * `openid` - A `String` representing the user's unique identifier in WeChat.
    ///
    /// # Returns
    ///
    /// * A `Result` containing a `UserInfoResponse` on success, or a boxed error on failure.
    ///
    /// # Errors
    ///
    /// * Returns an error if the access token is not found in Redis, if the HTTP
    /// request fails or returns a non-success status, or if the response cannot be
    /// deserialized into a `UserInfoResponse`.
    pub async fn get_userinfo(
        &self,
        openid: String,
    ) -> Result<UserInfoResponse, Box<dyn std::error::Error>> {
        let mut rdb = self.rdb_pool.get().await?;

        let access_token = match cmd("GET")
            .arg(&openid)
            .query_async::<String>(&mut rdb)
            .await
        {
            Ok(access_token) => access_token,
            Err(_) => {
                return Err("access token not found".into());
            }
        };

        let url = format!(
            "https://api.weixin.qq.com/sns/userinfo?access_token={}&openid={}",
            access_token, openid
        );

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        // 检查 HTTP 响应状态码
        if !status.is_success() {
            return Err(format!("HTTP error: {}", status).into());
        }

        // 处理微信 API 响应
        let response_text = response.text().await?;
        if let Ok(api_error) = serde_json::from_str::<BasicResponse>(&response_text) {
            return Err(format!(
                "Wechat API error: code={}, message={}",
                api_error.errcode, api_error.errmsg
            )
            .into());
        }

        // 尝试解析为 `UserInfoResponse`
        let user_info: UserInfoResponse = match serde_json::from_str(&response_text) {
            Ok(info) => info,
            Err(e) => {
                eprintln!(
                    "Failed to deserialize UserInfoResponse: {}, response: {}",
                    e, response_text
                );
                return Err(format!("Error decoding response body: {}", e).into());
            }
        };

        Ok(user_info)
    }
}
