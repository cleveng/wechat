pub struct MiniProgram {
    appid: String,
    app_secret: String,
}

impl MiniProgram {
    pub fn new(appid: String, app_secret: String) -> Self {
        MiniProgram { appid, app_secret }
    }
}
