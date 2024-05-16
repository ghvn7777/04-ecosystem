use std::time::Duration;

use axum::{routing::get, Router};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{debug, info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // file_appender 是 RollingFileAppender 类型，可以将 log 写入文件，不过默认是阻塞操作
    // daily 表示每天生成一个新的日志文件
    let file_appender = tracing_appender::rolling::daily("/tmp/logs", "ecosystem.log");
    // 将其变成非阻塞方式
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // with_span_events(FmtSpan::CLOSE) 表示在 span 结束时打印 span 的信息（也就是每个函数结束，每个函数都是一个 span）
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::DEBUG);

    let file = fmt::Layer::new()
        .with_writer(non_blocking)
        .pretty()
        .with_filter(LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:7654";
    let app = Router::new().route("/", get(index_handler));
    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

// 追踪异步函数，需要使用 #[instrument]
#[instrument]
async fn index_handler() -> &'static str {
    debug!("index_handler started");
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index_handler finished");
    ret
}

// 追踪异步函数，需要使用 #[instrument]
#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "task takes too long");
    "Hello, world!"
}

/*
  ## 启动时候第一条 log
  2024-05-16T17:04:35.715055Z  INFO axum_tracing: Starting server on 0.0.0.0:7654
    at examples/axum_tracing.rs:39

  ## 打印出进入到 index_handler 里面第一条 debug log（因为我们 console 里面设置的 log 等级是 debug）
  2024-05-16T17:04:37.793522Z DEBUG axum_tracing: index_handler started
    at examples/axum_tracing.rs:49
    in axum_tracing::index_handler

  ## 进入到 long_task 后 sleep，睡眠结束后打印出 warn log
  2024-05-16T17:04:37.918492Z  WARN axum_tracing: task takes too long, app.task_duration: 113
    at examples/axum_tracing.rs:61
    in axum_tracing::long_task
    in axum_tracing::index_handler

  ## long_task 结束后，离开了 long_task 函数的 span, 触发了 long_task 的 close log，打印用时以及 cpu 忙碌时间等信息
  2024-05-16T17:04:37.918787Z  INFO axum_tracing: close, time.busy: 439µs, time.idle: 113ms
    at examples/axum_tracing.rs:56
    in axum_tracing::long_task
    in axum_tracing::index_handler

  ## index_handler 调用完 long_task() 后，打印出 info 级别的 index_handler 的结束 log
  2024-05-16T17:04:37.918909Z  INFO axum_tracing: index_handler finished, http.status: 200
    at examples/axum_tracing.rs:52
    in axum_tracing::index_handler

  ## 离开 index_handler 函数的 span, 触发了 index_handler 的 close log，打印用时以及 cpu 忙碌时间等信息
  2024-05-16T17:04:37.918987Z  INFO axum_tracing: close, time.busy: 828µs, time.idle: 125ms
    at examples/axum_tracing.rs:47
    in axum_tracing::index_handler
*/
