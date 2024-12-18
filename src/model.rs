use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub openid: String,
    pub scope: String,
    pub unionid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
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
