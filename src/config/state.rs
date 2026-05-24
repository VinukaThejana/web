use super::ENV;
use envmode::EnvMode;
use redis::aio::MultiplexedConnection;
use resend_rs::Resend;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{sync::Arc, time::Duration};
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct AppState {
    rs: Resend,
    rd: redis::Client,
    rd_conn: Arc<OnceCell<MultiplexedConnection>>,
    db: Arc<OnceCell<DatabaseConnection>>,
    s3: Arc<OnceCell<aws_sdk_s3::Client>>,
}

impl AppState {
    pub async fn new() -> Self {
        let rs = Resend::new(&ENV.resend_api_key);
        let rd = redis::Client::open(&*ENV.redis_url).unwrap_or_else(|e| {
            log::error!("failed to open redis client: {:?}", e);
            std::process::exit(1);
        });

        Self {
            rs,
            rd,
            rd_conn: Arc::new(OnceCell::new()),
            db: Arc::new(OnceCell::new()),
            s3: Arc::new(OnceCell::new()),
        }
    }
}

impl AppState {
    pub fn resend(&self) -> &Resend {
        &self.rs
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

    pub async fn db(&self) -> &DatabaseConnection {
        self.db
            .get_or_init(|| async {
                let mut opt = ConnectOptions::new(&*ENV.db_url);
                opt.max_lifetime(Duration::from_secs(8))
                    .idle_timeout(Duration::from_secs(5))
                    .sqlx_logging(true)
                    .sqlx_logging_level(if EnvMode::is_prd(&ENV.environment) {
                        log::LevelFilter::Trace
                    } else {
                        log::LevelFilter::Info
                    })
                    .set_schema_search_path(&*ENV.db_schema);

                Database::connect(opt).await.unwrap_or_else(|e| {
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
            if let Err(err) = db.close().await {
                log::error!("failed to close database connection: {:?}", err);
            }
        }
    }
}
