use crate::{
    model::{AccessTokenResponse, AuthResponse, UserInfoResponse},
    WechatType,
};
use async_trait::async_trait;
use reqwest::Client;
use std::error::Error;
use url::{form_urlencoded, Url};

pub struct OpenPlatform {
    appid: String,
    app_secret: String,
}

impl OpenPlatform {
    pub fn new(appid: String, app_secret: String) -> Self {
        OpenPlatform { appid, app_secret }
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

    async fn get_access_token(
        &self,
        code: String,
    ) -> Result<crate::model::AccessTokenResponse, Box<dyn std::error::Error>> {
        let url = format!(
        "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code", self.appid, self.app_secret, code);

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let access_token: AccessTokenResponse = response.json::<AccessTokenResponse>().await?;

        Ok(access_token)
    }

    // https://developers.weixin.qq.com/doc/oplatform/Website_App/WeChat_Login/Authorized_Interface_Calling_UnionID.html
    async fn refresh_access_token(
        &self,
        appid: String,
        refresh_token: String,
    ) -> Result<AccessTokenResponse, Box<dyn Error>> {
        let url = format!(
        "https://api.weixin.qq.com/sns/oauth2/refresh_token?appid={}&grant_type=refresh_token&refresh_token={}",
        appid, refresh_token
    );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let access_token: AccessTokenResponse = response.json::<AccessTokenResponse>().await?;

        Ok(access_token)
    }

    async fn check_access_token(
        &self,
        openid: String,
        access_token: String,
    ) -> Result<AuthResponse, Box<dyn Error>> {
        let url = format!(
            "https://api.weixin.qq.com/sns/auth?access_token={}&openid={}",
            access_token, openid
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let auth: AuthResponse = response.json::<AuthResponse>().await?;
        Ok(auth)
    }

    async fn get_user_info(
        &self,
        openid: String,
        access_token: String,
    ) -> Result<UserInfoResponse, Box<dyn Error>> {
        let url = format!(
            "https://api.weixin.qq.com/sns/userinfo?access_token={}&openid={}",
            access_token, openid
        );

        let client = Client::new();
        let response = client.get(&url).send().await?;
        let user_info: UserInfoResponse = response.json::<UserInfoResponse>().await?;
        Ok(user_info)
    }
}
