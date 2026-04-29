mod config;
mod db;
mod error;
mod models;
mod auth;
mod auth_supabase;
mod supabase;
mod workspaces;
mod channels;
mod conversations;
mod messages;
mod api;
mod test_connection;
mod redis_client;
mod cache_service;
mod test_redis;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use config::Config;
use db::Database;
use supabase::SupabaseClient;
use auth_supabase::SupabaseAuthService;
use redis_client::RedisClient;
use cache_service::CacheService;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    // Load configuration
    let config = Config::from_env()?;
    let server_addr = config.server_addr.clone();
    
    // Test Supabase connection first
    test_connection::test_supabase_connection().await?;
    
    // Connect to Supabase database
    tracing::info!("Connecting to Supabase database...");
    let db = Database::new(&config.database_url).await?;
    tracing::info!("Database connection established successfully");

    // Initialize Supabase client
    tracing::info!("Initializing Supabase client...");
    let supabase_client = SupabaseClient::new(&config);
    tracing::info!("Supabase client initialized");

    // Initialize Supabase Auth service
    let auth_service = SupabaseAuthService::new(supabase_client.clone(), db.clone());
    tracing::info!("Supabase Auth service initialized");

    // Initialize Redis client
    tracing::info!("Connecting to Redis...");
    let redis_client = RedisClient::new(&config.redis_url).await?;
    tracing::info!("Redis connection established successfully");

    // Test Redis connection
    let ping_result = redis_client.test_connection().await?;
    tracing::info!("Redis PING response: {}", ping_result);

    // Initialize Cache service
    let cache_service = CacheService::new(redis_client.clone());
    tracing::info!("Cache service initialized");

    // Test Redis functionality
    test_redis::test_redis_connection().await?;
    
    tracing::info!("Starting server on {}", server_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(supabase_client.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(redis_client.clone()))
            .app_data(web::Data::new(cache_service.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(Cors::permissive())
            .configure(api::configure_routes)
    })
    .bind(&server_addr)?
    .run()
    .await?;
    
    Ok(())
}
