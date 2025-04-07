use super::ENV;
use envmode::EnvMode;
use redis::{Client as RedisClient, RedisError, aio::MultiplexedConnection};
use resend_rs::Resend;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub rs: Resend,
    pub rd: RedisClient,
}

impl AppState {
    pub async fn get_redis_conn<E>(&self) -> Result<MultiplexedConnection, E>
    where
        RedisError: Into<E>,
    {
        self.rd
            .get_multiplexed_async_connection()
            .await
            .map_err(Into::into)
    }
}

impl AppState {
    pub async fn new() -> Self {
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

        let db: DatabaseConnection = Database::connect(opt).await.unwrap_or_else(|e| {
            log::error!("failed to connect to database: {:?}", e);
            std::process::exit(1);
        });

        let rs = Resend::new(&ENV.resend_api_key);
        let rd = RedisClient::open(&*ENV.redis_url).unwrap_or_else(|e| {
            log::error!("failed to connect to redis: {:?}", e);
            std::process::exit(1);
        });

        Self { db, rs, rd }
    }
}

impl AppState {
    pub async fn close(self) {
        if let Err(e) = self.db.close().await {
            log::error!("failed to close database connection: {:?}", e);
        }
    }
}
