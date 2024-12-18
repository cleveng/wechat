pub struct OfficialAccount {
    appid: String,
    app_secret: String,
}

impl OfficialAccount {
    pub fn new(appid: String, app_secret: String) -> Self {
        OfficialAccount { appid, app_secret }
    }
}
