use crate::OfficialAccount;

use super::core::UserInfoResponse;

impl OfficialAccount {
    pub async fn get_user_by_open_id(
        &self,
        open_id: &str,
    ) -> Result<UserInfoResponse, Box<dyn std::error::Error>> {
        let token = self.token().await?;

        let url = format!(
            "https://api.weixin.qq.com/cgi-bin/user/info?access_token={}&openid={}&lang=zh_CN",
            token, open_id
        );
        let response = match self.client.get(url).send().await {
            Ok(r) => r,
            Err(e) => {
                println!("json err: {:#?}", e);
                return Err(e.into());
            }
        };

        let result = match response.json::<UserInfoResponse>().await {
            Ok(r) => r,
            Err(e) => {
                println!("json err: {:#?}", e);
                return Err(e.into());
            }
        };

        Ok(result)
    }
}
