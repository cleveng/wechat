use crate::OfficialAccount;

use super::core::BasicResponse;

pub(crate) const DELETE_MENU_URL: &str = "https://api.weixin.qq.com/cgi-bin/menu/delete?access_token=";

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{Config, OfficialAccount};

    #[tokio::test]
    async fn delete_menu() {
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
        let result = account.delete_menu().await;
        println!("url: {:#?}", result);
    }
}

impl OfficialAccount {
    /// [Deletes all custom menus for the official account](https://developers.weixin.qq.com/doc/offiaccount/Custom_Menus/Deleting_Custom-Defined_Menu.html)
    ///
    /// # Returns
    ///
    /// * A `Result` containing a `String` with the value `"ok"` on success, or a boxed
    ///   error on failure.
    ///
    /// # Errors
    ///
    /// * Returns an error if the HTTP request fails or returns a non-success status,
    ///   or if the response cannot be deserialized into a `BasicResponse`.
    pub async fn delete_menu(&self) -> Result<String, Box<dyn std::error::Error>> {
        let token = self.token().await?;

        let url = format!("{}{}", DELETE_MENU_URL, token);
        let response = self.client.post(url).send().await?;
        if let Err(err) = response.json::<BasicResponse>().await {
            println!("json err: {:#?}", err);
            return Err(err.into());
        }

        Ok("ok".to_string())
    }
}
