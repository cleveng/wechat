use crate::{
    model::{AccessTokenResponse, AuthResponse, UserInfoResponse},
    WechatType,
};
use async_trait::async_trait;
use deadpool_redis::redis::cmd;
use deadpool_redis::{Config, Pool, Runtime};
use reqwest::Client;
use std::error::Error;
use std::sync::Arc;
use url::{form_urlencoded, Url};

pub struct OpenPlatform {
    appid: String,
    app_secret: String,
    rdb: Option<Arc<Pool>>,
}

impl OpenPlatform {
    pub fn new(appid: String, app_secret: String, cfg: Option<String>) -> Self {
        let rdb = match cfg {
            Some(cfg) => {
                let pool_config = Config::from_url(cfg);
                let pool = pool_config.create_pool(Some(Runtime::Tokio1)).unwrap();
                Some(Arc::new(pool))
            }
            None => None,
        };
        OpenPlatform {
            appid,
            app_secret,
            rdb,
        }
    }
}

#[async_trait]
impl WechatType for OpenPlatform {
    fn get_redirect_url(&self, redirect_uri: String, state: Option<String>) -> String {
        let mut url = Url::parse("https://open.weixin.qq.com/connect/qrconnect").unwrap();

        let _state = state.unwrap_or("".to_string());
        let query = form_urlencoded::Serializer::new(String::new())
            .append_pair("appid", self.appid.as_ref())
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", "snsapi_login")
            .append_pair("state", &_state)
            .append_pair("lang", "")
            .finish();

        url.set_query(Some(&query));

        url.to_string()
    }

    // https://developers.weixin.qq.com/doc/oplatform/Website_App/WeChat_Login/Authorized_Interface_Calling_UnionID.html
    async fn get_access_token(
        &self,
        code: String,
    ) -> Result<crate::model::AccessTokenResponse, Box<dyn std::error::Error>> {
        let url = format!(
        "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code", self.appid, self.app_secret, code);

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let at: AccessTokenResponse = response.json::<AccessTokenResponse>().await?;

        if let Some(pool) = &self.rdb {
            let mut rdb = pool.get().await.unwrap();
            // openid -> access_token
            cmd("SETEX")
                .arg(&at.openid)
                .arg(2 * 60 * 60)
                .arg(&at.access_token)
                .query_async::<()>(&mut rdb)
                .await
                .unwrap();

            // appid -> refresh_token
            cmd("SETEX")
                .arg(&self.appid)
                .arg(24 * 60 * 60 * 7)
                .arg(&at.refresh_token)
                .query_async::<()>(&mut rdb)
                .await
                .unwrap()
        }

        Ok(at)
    }

    // https://developers.weixin.qq.com/doc/oplatform/Website_App/WeChat_Login/Authorized_Interface_Calling_UnionID.html
    async fn refresh_access_token(
        &self,
        appid: String,
    ) -> Result<AccessTokenResponse, Box<dyn Error>> {
        let mut rdb = match &self.rdb {
            Some(rdb) => rdb.get().await.unwrap(),
            None => {
                return Err("redis pool is not initialized".into());
            }
        };

        let refresh_token = match cmd("GET").arg(&appid).query_async::<String>(&mut rdb).await {
            Ok(refresh_token) => refresh_token,
            Err(_) => {
                return Err("redis key not found".into());
            }
        };

        let url = format!(
        "https://api.weixin.qq.com/sns/oauth2/refresh_token?appid={}&grant_type=refresh_token&refresh_token={}",
        appid, refresh_token
    );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let access_token: AccessTokenResponse = response.json::<AccessTokenResponse>().await?;

        Ok(access_token)
    }

    async fn check_access_token(&self, openid: String) -> Result<AuthResponse, Box<dyn Error>> {
        let mut rdb = match &self.rdb {
            Some(rdb) => rdb.get().await.unwrap(),
            None => {
                return Err("redis pool is not initialized".into());
            }
        };

        let access_token = match cmd("GET")
            .arg(&openid)
            .query_async::<String>(&mut rdb)
            .await
        {
            Ok(access_token) => access_token,
            Err(_) => {
                return Err("redis key not found".into());
            }
        };

        let url = format!(
            "https://api.weixin.qq.com/sns/auth?access_token={}&openid={}",
            access_token, openid
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let auth: AuthResponse = response.json::<AuthResponse>().await?;
        Ok(auth)
    }

    async fn get_user_info(&self, openid: String) -> Result<UserInfoResponse, Box<dyn Error>> {
        let mut rdb = match &self.rdb {
            Some(rdb) => rdb.get().await.unwrap(),
            None => {
                return Err("redis pool is not initialized".into());
            }
        };

        let access_token = match cmd("GET")
            .arg(&openid)
            .query_async::<String>(&mut rdb)
            .await
        {
            Ok(access_token) => access_token,
            Err(_) => {
                return Err("redis key not found".into());
            }
        };

        let url = format!(
            "https://api.weixin.qq.com/sns/userinfo?access_token={}&openid={}",
            access_token, openid
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let status = response.status();

        // 检查 HTTP 响应状态码
        if !status.is_success() {
            return Err(format!("HTTP error: {}", status).into());
        }

        // 处理微信 API 响应
        let response_text = response.text().await?;
        if let Ok(api_error) = serde_json::from_str::<AuthResponse>(&response_text) {
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
