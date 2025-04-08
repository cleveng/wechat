use std::collections::HashMap;

use crate::OfficialAccount;

use super::core::BasicResponse;

const CLEAR_QUOTA_URL: &str = "https://api.weixin.qq.com/cgi-bin/clear_quota?access_token=";

#[cfg(test)]
mod tests {

    use crate::OfficialAccount;
    use std::env;

    #[tokio::test]
    async fn get_qr_ticket() {
        dotenv::dotenv().ok();

        let appid = env::var("APPID").expect("appid not set");
        let app_secret = env::var("APP_SECRET").expect("app secret not set");
        let cfg = env::var("REDIS_URL").expect("redis url not set");

        let account = OfficialAccount::new(appid, app_secret, cfg);

        let at = account.clear_quota().await;
        println!("get_qr_ticket: {:#?}", at);
    }
}

impl OfficialAccount {
    /// [清空api的调用quota](https://developers.weixin.qq.com/doc/offiaccount/openApi/clear_quota.html)
    pub async fn clear_quota(&self) -> Result<(), Box<dyn std::error::Error>> {
        let token = self.token().await?;

        let mut params = HashMap::new();
        params.insert("appid".to_string(), self.appid.clone());

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
