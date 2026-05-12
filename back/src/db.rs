use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[derive(Debug)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        tracing::info!("Attempting to connect to database: {}", database_url);
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        tracing::info!("Database pool created successfully");
        
        // Test the connection
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await?;
        
        tracing::info!("Database connection test passed");

        Ok(Database { pool })
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pool: self.pool.clone(),
        }
    }
}
