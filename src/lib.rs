pub mod model;
pub mod open_platform;

use async_trait::async_trait;
use model::{AccessTokenResponse, AuthResponse, UserInfoResponse};
use std::error::Error;

#[async_trait]
pub trait WechatType {
    fn get_redirect_url(&self, redirect_uri: String, state: Option<String>) -> String;
    async fn get_access_token(&self, code: String) -> Result<AccessTokenResponse, Box<dyn Error>>;
    async fn refresh_access_token(
        &self,
        appid: String,
        refresh_token: String,
    ) -> Result<AccessTokenResponse, Box<dyn Error>>;
    async fn check_access_token(
        &self,
        openid: String,
        access_token: String,
    ) -> Result<AuthResponse, Box<dyn Error>>;
    async fn get_user_info(
        &self,
        openid: String,
        access_token: String,
    ) -> Result<UserInfoResponse, Box<dyn Error>>;
}

pub struct Wechat<T: WechatType> {
    wechat_type: T,
}

impl<T: WechatType> Wechat<T> {
    pub fn new(wechat_type: T) -> Self {
        Wechat { wechat_type }
    }

    pub fn get_redirect_url(&self, redirect_uri: String, state: Option<String>) -> String {
        self.wechat_type.get_redirect_url(redirect_uri, state)
    }

    pub async fn get_access_token(
        &self,
        code: String,
    ) -> Result<AccessTokenResponse, Box<dyn Error>> {
        self.wechat_type.get_access_token(code).await
    }

    pub async fn refresh_access_token(
        &self,
        appid: String,
        refresh_token: String,
    ) -> Result<AccessTokenResponse, Box<dyn Error>> {
        self.wechat_type
            .refresh_access_token(appid, refresh_token)
            .await
    }

    pub async fn check_access_token(
        &self,
        openid: String,
        access_token: String,
    ) -> Result<AuthResponse, Box<dyn Error>> {
        self.wechat_type
            .check_access_token(openid, access_token)
            .await
    }
}
