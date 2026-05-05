pub mod schema;

use anyhow::Result;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use std::env;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection = AsyncPgConnection;

pub async fn create_db_pool() -> Result<DbPool> {
    let database_url = env::var("DATABASE_URL")?;
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let pool = Pool::builder(config)
        .build()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(pool)
}
