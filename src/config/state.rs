use super::ENV;
use redis::aio::MultiplexedConnection;
use reqwest::redirect::Policy;
use resend_rs::Resend;
use std::{str::FromStr, sync::Arc, time::Duration};
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct AppState {
    rs: Resend,
    rd: redis::Client,
    rd_conn: Arc<OnceCell<MultiplexedConnection>>,
    db: Arc<OnceCell<sqlx::PgPool>>,
    s3: Arc<OnceCell<aws_sdk_s3::Client>>,
    http: reqwest::Client,
    http_no_redirect: reqwest::Client,
}

impl AppState {
    pub async fn new() -> Self {
        let rs = Resend::new(&ENV.resend_api_key);
        let rd = redis::Client::open(&*ENV.redis_url).unwrap_or_else(|e| {
            log::error!("failed to open redis client: {:?}", e);
            std::process::exit(1);
        });

        let http = reqwest::Client::new();
        let http_no_redirect = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("failed to build no-redirect reqwest client");

        Self {
            rs,
            rd,
            rd_conn: Arc::new(OnceCell::new()),
            db: Arc::new(OnceCell::new()),
            s3: Arc::new(OnceCell::new()),
            http,
            http_no_redirect,
        }
    }
}

impl AppState {
    pub fn resend(&self) -> &Resend {
        &self.rs
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn http_no_redirect(&self) -> &reqwest::Client {
        &self.http_no_redirect
    }

    pub async fn redis(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        Ok(self
            .rd_conn
            .get_or_init(|| async {
                self.rd
                    .get_multiplexed_async_connection()
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("failed to connect to redis: {:?}", e);
                        std::process::exit(1);
                    })
            })
            .await
            .clone())
    }

    pub async fn db(&self) -> &sqlx::PgPool {
        self.db
            .get_or_init(|| async {
                let mut options = sqlx::postgres::PgConnectOptions::from_str(&ENV.db_url)
                    .unwrap_or_else(|e| {
                        log::error!("failed to parse database url: {:?}", e);
                        std::process::exit(1);
                    });
                options = options.options([("search_path", ENV.db_schema.to_string())]);

                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(1)
                    .min_connections(1)
                    .idle_timeout(Duration::from_secs(5))
                    .max_lifetime(Duration::from_secs(8))
                    .connect_with(options)
                    .await
                    .unwrap_or_else(|e| {
                        log::error!("failed to connect to database: {:?}", e);
                        std::process::exit(1);
                    })
            })
            .await
    }

    pub async fn s3(&self) -> &aws_sdk_s3::Client {
        self.s3
            .get_or_init(|| async {
                let config = aws_config::from_env()
                    .endpoint_url(&*ENV.cloudflare_endpoint)
                    .credentials_provider(aws_sdk_s3::config::Credentials::new(
                        &*ENV.cloudflare_access_key_id,
                        &*ENV.cloudflare_access_key_secret,
                        None,
                        None,
                        "R2",
                    ))
                    .region("auto")
                    .load()
                    .await;

                aws_sdk_s3::Client::new(&config)
            })
            .await
    }
}

impl AppState {
    pub async fn close(self) {
        if let Some(db) = Arc::try_unwrap(self.db).ok().and_then(|c| c.into_inner()) {
            db.close().await;
        }
    }
}
