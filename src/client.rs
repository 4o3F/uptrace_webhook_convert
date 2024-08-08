use anyhow::Result;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BarkNotificationLevel {
    Active,
    TimeSensitive,
    Passive,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarkPostBody {
    pub title: Option<String>,
    pub body: Option<String>,
    pub level: Option<BarkNotificationLevel>,
    pub badge: Option<i32>,
    pub auto_copy: Option<bool>,
    pub copy: Option<String>,
    pub sound: Option<String>,
    pub icon: Option<String>,
}

pub async fn bark_post(bark_url: &String, body: BarkPostBody) -> Result<()> {
    let body = serde_json::to_string(&body)?;
    match reqwest::Client::new()
        .post(bark_url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!("消息推送成功");
            Ok(())
        }
        Err(e) => {
            tracing::error!("消息推送失败: {e}");
            Err(e.into())
        }
    }
}
