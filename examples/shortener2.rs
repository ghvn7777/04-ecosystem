use anyhow::Result;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const LISTEN_ADDR: &str = "127.0.0.1:9876";

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Error, Debug)]
pub enum ShortenError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ShortenError {
    fn into_response(self) -> Response {
        let info = self.to_string();
        warn!("{:?}", info);

        // use client-facing message
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(info.into())
            .unwrap()
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let url = "postgres://postgres:postgres@localhost:5432/shortener";
    let state = AppState::try_new(url).await?;

    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {}", LISTEN_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>, // body 的 extractor 要放到最后，不然会报错。记住就行， path 没影响
) -> Result<impl IntoResponse, ShortenError> {
    let id = state.shorten(&data.url).await?;
    let body = Json(ShortenRes {
        url: format!("http://{}/{}", LISTEN_ADDR, id),
    });
    Ok((StatusCode::CREATED, body))
}

async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ShortenError> {
    let url = state.get_url(&id).await?;
    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, url.parse().unwrap());
    // PERMANENT_REDIRECT = 308
    // 这个状态码被认为是实验性的，但它的语义与301永久重定向相同。
    // 308和301重定向的唯一区别是是否可以修改HTTP方法。
    // 301重定向允许用户代理修改使用的HTTP方法，而308状态码则意味着重定向的HTTP请求方法是不可改变的。
    // 308 HTTP状态码是相当新的，因为它在2015年才被引入。
    Ok((StatusCode::PERMANENT_REDIRECT, headers))
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        // 注意这里设置了 url 也不能重复
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { db: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String, ShortenError> {
        let mut id = nanoid!(6); // url 友好的 id，我们设置长度为 6
                                 // 这里 url 冲突的话，就把冲突的 url 更新为新的 url（相当于啥也没做，只是为了找到冲突那行），然后返回冲突的 id
                                 // UrlRecord 的字段都设置了 #[sqlx(default)]，所以这里即使是只返回 id，url 也会被填充为默认值返回
                                 // info!("---id: {}", id);
        loop {
            let id_res: Result<UrlRecord, _> = sqlx::query_as("SELECT id FROM urls WHERE id = $1")
                .bind(id.clone())
                .fetch_one(&self.db)
                .await;
            if id_res.is_err() {
                break;
            }
            id = nanoid!(6);
            // info!("huan ---id: {}", id);
        }

        let ret: UrlRecord = sqlx::query_as(
            "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
        )
        .bind(&id)
        .bind(url)
        .fetch_one(&self.db)
        .await?;

        Ok(ret.id)
    }

    async fn get_url(&self, id: &str) -> Result<String, ShortenError> {
        let ret: UrlRecord = sqlx::query_as("SELECT url FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await?;

        Ok(ret.url)
    }
}
