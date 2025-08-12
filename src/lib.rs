mod constants;
pub mod official_account;

use deadpool_redis::{Pool, Runtime};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct OfficialAccount {
    config: Config,
    rdb_pool: Arc<Pool>,
    client: Client,
}

pub struct Config {
    pub appid: String,
    pub app_secret: String,
    pub token: String,
    pub encoding_aes_key: Option<String>,
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
    pub fn new(conf: Config, redis_url: String) -> Self {
        let pool_config = deadpool_redis::Config::from_url(redis_url);

        let rdb_pool = match pool_config.create_pool(Some(Runtime::Tokio1)) {
            Ok(pool) => Arc::new(pool),
            Err(err) => {
                panic!("Failed to create Redis pool: {}", err);
            }
        };

        OfficialAccount {
            config: conf,
            rdb_pool,
            client: Client::new(),
        }
    }
}
