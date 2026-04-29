mod config;
mod db;
mod error;
mod models;
mod auth;
mod workspaces;
mod channels;
mod conversations;
mod messages;
mod websocket;
mod api;

use actix_web::{web, App, HttpServer, middleware};
use actix::Addr;
use config::Config;
use db::Database;
use websocket::ChatServer;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logger
    env_logger::init();
    
    // Load configuration
    let config = Config::from_env()?;
    
    // Initialize database pool
    let db = Database::new(&config.database_url).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db.pool).await?;
    
    // Start chat server for WebSocket connections
    let chat_server = websocket::start_chat_server(db.clone()).await;
    
    tracing::info!("Starting server on {}", config.server_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(chat_server.clone()))
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Cors::permissive())
            .configure(api::configure_routes)
    })
    .bind(&config.server_addr)?
    .run()
    .await?;
    
    Ok(())
}
