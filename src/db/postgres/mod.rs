use anyhow::Result;
use diesel_async::{
    pooled_connection::bb8::Pool,
    pooled_connection::AsyncDieselConnectionManager,
    AsyncPgConnection
};

pub mod schema;

pub type DbPool = Pool<AsyncPgConnection>;


pub async fn new_pool(database_url: &str) -> Result<DbPool> {
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

    let pool = Pool::builder()
        .max_size(20)
        .connection_timeout(std::time::Duration::from_secs(10))
        .build(config)
        .await.map_err(|e| anyhow::anyhow!("Failed to create DB pool: {}", e))?;
    Ok(pool)
}