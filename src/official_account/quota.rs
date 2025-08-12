use std::collections::HashMap;

use crate::OfficialAccount;

use super::core::BasicResponse;

pub(crate) const CLEAR_QUOTA_URL: &str = "https://api.weixin.qq.com/cgi-bin/clear_quota?access_token=";

#[cfg(test)]
mod tests {

    use crate::{Config, OfficialAccount};
    use std::env;

    #[tokio::test]
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

        let at = account.clear_quota().await;
        println!("get_qr_ticket: {:#?}", at);
    }
}

impl OfficialAccount {
    /// [清空api的调用quota](https://developers.weixin.qq.com/doc/offiaccount/openApi/clear_quota.html)
    pub async fn clear_quota(&self) -> Result<(), Box<dyn std::error::Error>> {
        let token = self.token().await?;

        let mut params = HashMap::new();
        params.insert("appid".to_string(), self.config.appid.clone());

        let url = format!("{}{}", CLEAR_QUOTA_URL, token);
        let response = match self.client.post(url).json(&params).send().await {
            Ok(r) => r,
            Err(e) => {
                println!("json err: {:#?}", e);
                return Err(e.into());
            }
        };

        if let Err(err) = response.json::<BasicResponse>().await {
            println!("json err: {:#?}", err);
            return Err(err.into());
        }

        Ok(())
    }
}
