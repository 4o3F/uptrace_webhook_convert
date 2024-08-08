use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub port: String,
    pub bark_notify_urls: Vec<String>,
}

pub fn init() -> Config {
    let config_file = match fs::read_to_string("config.json") {
        Ok(config_file) => config_file,
        Err(e) => {
            tracing::error!("读取配置文件失败 {}", e);
            std::process::exit(1);
        }
    };
    let config: Config = match serde_json::from_str(&config_file) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("配置文件解析失败 {}", e);
            std::process::exit(1)
        }
    };

    tracing::info!("配置文件加载成功");
    tracing::info!("Bark通知URL共 {} 个", config.bark_notify_urls.len());

    config
}
