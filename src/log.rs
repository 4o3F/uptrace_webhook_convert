use anyhow::{anyhow, Result};
use tracing_appender::{non_blocking, rolling::Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt};

pub fn init() -> Result<non_blocking::WorkerGuard> {
    let console_layer = fmt::layer().pretty().with_writer(std::io::stderr);

    let file_appender = tracing_appender::rolling::Builder::new()
        .filename_suffix("app.log")
        .rotation(Rotation::DAILY)
        .max_log_files(30)
        .build("./logs")?;
    let (non_blocking_appender, guard) = non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer);

    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_) => {
            tracing::info!("初始化日志成功");
            return Ok(guard);
        }
        Err(e) => {
            return Err(anyhow!("初始化日志失败: {e}"));
        }
    }
}
