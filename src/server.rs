use anyhow::Result;
use axum::{routing::post, Json, Router};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::{signal, task::JoinSet};

use crate::{
    client::{self, BarkPostBody},
    CONFIG,
};

#[derive(Deserialize, Clone, Debug)]
struct UptraceAlert {
    pub id: Option<String>,
    pub url: Option<String>,
    pub name: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub alert_type: Option<String>,
    pub state: Option<String>,
    #[serde(rename(deserialize = "createdAt"))]
    pub created_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct UptraceWebhook {
    pub id: Option<String>,
    #[serde(rename(deserialize = "eventName"))]
    pub event_name: Option<String>,
    pub payload: serde_json::Value,
    #[serde(rename(deserialize = "createdAt"))]
    pub created_at: Option<String>,
    pub alert: UptraceAlert,
}

pub async fn init() -> Result<()> {
    let app = Router::new().route("/", post(webhook_handler));
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &CONFIG.port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法监听Ctrl+C信号");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("信号处理器安装失败")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("关闭中....");
}

async fn webhook_handler(Json(payload): Json<UptraceWebhook>) -> StatusCode {
    tracing::debug!("Webhook payload: {:?}", payload);
    let bark_post_body = BarkPostBody {
        title: Some(format!(
            "{} {} 事件",
            match payload.event_name.unwrap().as_str() {
                "created" => "新增".to_string(),
                "status-changed" => "更新".to_string(),
                "recurring" => "重复出现".to_string(),
                _ => "其他动作".to_string(),
            },
            match payload.alert.alert_type.unwrap().as_str() {
                "metric" => "监控".to_string(),
                "error" => "错误".to_string(),
                _ => "其他".to_string(),
            }
        )),
        body: Some(format!(
            "{}\n{}",
            payload.alert.name.unwrap(),
            payload.alert.created_at.unwrap()
        )),
        level: Some(crate::client::BarkNotificationLevel::TimeSensitive),
        badge: None,
        auto_copy: Some(true),
        copy: Some(payload.alert.url.unwrap()),
        sound: Some("minuet".to_string()),
        icon: None,
    };

    let notify_urls = &CONFIG.bark_notify_urls;

    let mut join_set = JoinSet::new();

    for url in notify_urls {
        let bark_post_body = bark_post_body.clone();
        join_set.spawn(async move { client::bark_post(url, bark_post_body).await });
    }

    while let Some(res) = join_set.join_next().await {
        let out = match res {
            Ok(res) => res,
            Err(e) => {
                tracing::error!("服务器内部错误 {}", e);
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        };

        if let Err(e) = out {
            tracing::error!("Bark请求发送失败 {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    StatusCode::OK
}
