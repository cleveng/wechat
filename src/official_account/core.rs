use crate::OfficialAccount;

use deadpool_redis::redis::cmd;
use serde::{Deserialize, Serialize};
use url::{Url, form_urlencoded};

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

        let url = account.get_redirect_url(redirect_uri, None);
        println!("url: {:#?}", url);
    }

    #[tokio::test]
    async fn get_oauth2_token() {
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
        let at = account
            .get_oauth2_token("011OrEIa1C5KpJ0slBGa1i73tY3OrEI2".to_string())
            .await;

        println!("get_oauth2_token: {:#?}", at.unwrap());
    }

    #[tokio::test]
    async fn get_userinfo() {
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
        let at = account
            .get_userinfo("oVwE26e3c3jS8M63WeRgKHZX-z7Y".to_string())
            .await;

        println!("get_oauth2_token: {:#?}", at.unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    pub openid: String,
    scope: String,
    pub unionid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BasicResponse {
    pub errcode: i64,
    pub errmsg: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoResponse {
    pub openid: String,
    pub nickname: String,
    pub sex: i64,
    pub province: String,
    pub city: String,
    pub country: String,
    pub headimgurl: String,
    pub privilege: Vec<String>,
    pub unionid: Option<String>,
}

impl OfficialAccount {
    /// [Generates a URL that can be used to redirect the user to the WeChat authorization page.](https://developers.weixin.qq.com/doc/offiaccount/Basic_Information/get_oauth2_token.html)
    ///
    ///
    /// The user will be redirected to the `redirect_uri` after authorization.
    /// The `state` parameter is optional and can be used to store
    /// arbitrary data that will be passed back to your application after
    /// authorization.
    ///
    /// # Examples
    ///
    pub fn get_redirect_url(&self, redirect_uri: String, state: Option<String>) -> String {
        let mut url = Url::parse("https://open.weixin.qq.com/connect/oauth2/authorize").unwrap();
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("appid", &self.config.appid)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "snsapi_base")
            .append_pair("state", state.unwrap_or("".to_string()).as_ref())
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
            .arg(&at.openid)
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
