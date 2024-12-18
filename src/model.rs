use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AccessTokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    openid: String,
    scope: String,
    unionid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthResponse {
    errcode: i64,
    errmsg: String,
}

#[derive(Debug, Deserialize)]
pub struct UserInfoResponse {
    openid: String,
    nickname: String,
    sex: i64,
    province: String,
    city: String,
    country: String,
    headimgurl: String,
    privilege: Vec<String>,
    unionid: Option<String>,
}
