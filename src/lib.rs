pub mod constants;
pub mod official_account;

use deadpool_redis::{Config, Pool, Runtime};
use std::sync::Arc;

pub struct OfficialAccount {
    appid: String,
    app_secret: String,
    rdb_pool: Arc<Pool>,
}

impl OfficialAccount {
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
        }
    }
}
