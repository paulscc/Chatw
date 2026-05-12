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
mod websocket;

use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use config::Config;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    // Load configuration
    let config = Config::from_env()?;
    let server_addr = config.server_addr.clone();
    
        
    // Skip database connection completely for API-only mode
    tracing::warn!("Starting in API-only mode - no direct PostgreSQL connection");
    tracing::info!("Backend will use Redis + Supabase REST API");
    tracing::info!("WebSocket and real-time features will work through Redis");
    
    // TEMPORARY: Skip database connection completely
    tracing::warn!("TEMPORARY: Running without database connection");
    tracing::info!("Server will use Redis + Supabase REST API only");
    tracing::info!("WebSocket and real-time features will work through Redis");
    tracing::info!("To enable full functionality, configure PostgreSQL connection");
    
    // NEW VERSION: Skip database connection completely
    tracing::warn!("NEW VERSION: Running without database connection");
    tracing::info!("Server will use Redis + Supabase REST API only");
    tracing::info!("WebSocket and real-time features will work through Redis");
    tracing::info!("This version does not require PostgreSQL connection");
    
    // Skip database connection completely - this version works with API only
    tracing::info!("Skipping PostgreSQL connection - using API-only mode");

    // NEW VERSION: Minimal server test - no dependencies
    tracing::warn!("NEW VERSION: Minimal server test - no dependencies");
    tracing::info!("Starting minimal server to test basic functionality");
    
    tracing::info!("Starting server on {}", server_addr);
    
    HttpServer::new(move || {
        let redis_url = config.redis_url.clone();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(Cors::permissive())
            .app_data(web::Data::new(redis_url))
            .route("/health", web::get().to(|| async { "OK" }))
            .route("/", web::get().to(|| async { "Backend is running!" }))
            .route("/ws/{room_code}", web::get().to(websocket::index))
    })
    .bind(&server_addr)?
    .run()
    .await?;
    
    Ok(())
}
