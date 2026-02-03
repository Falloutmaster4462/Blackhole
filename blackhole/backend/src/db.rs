use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    // Read migration file
    let migration_sql = include_str!("../migrations/001_init.sql");

    // Execute migration using raw SQL (not prepared statement)
    // This allows multiple statements in one execution
    let mut conn = pool.acquire().await?;
    sqlx::raw_sql(migration_sql)
        .execute(&mut *conn)
        .await?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}