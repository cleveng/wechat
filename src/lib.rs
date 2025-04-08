pub mod constants;
pub mod official_account;

use deadpool_redis::{Config, Pool, Runtime};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct OfficialAccount {
    appid: String,
    app_secret: String,
    rdb_pool: Arc<Pool>,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub access_token: String,
    expires_in: u64,
}

impl OfficialAccount {
    /// Creates a new instance of the OfficialAccount struct.
    ///
    /// # Arguments
    ///
    /// * `appid` - The appid of the WeChat application.
    /// * `app_secret` - The app secret of the WeChat application.
    /// * `cfg` - The URL of the Redis database connection string.
    ///
    /// # Returns
    ///
    /// A new instance of the OfficialAccount struct.
    pub fn new(appid: String, app_secret: String, cfg: String) -> Self {
        let pool_config = Config::from_url(cfg);

        let rdb_pool = match pool_config.create_pool(Some(Runtime::Tokio1)) {
            Ok(pool) => Arc::new(pool),
            Err(err) => {
                panic!("Failed to create Redis pool: {}", err);
            }
        };

        OfficialAccount {
            appid,
            app_secret,
            rdb_pool,
            client: Client::new(),
        }
    }
}
