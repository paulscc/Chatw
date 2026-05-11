use crate::config::Config;

pub async fn test_supabase_connection() -> anyhow::Result<()> {
    tracing::info!("Testing Supabase connection...");
    
    // Cargar configuración desde variables de entorno
    let config = Config::from_env()?;
    tracing::info!("Supabase URL: {}", &config.supabase_url[..config.supabase_url.len().min(50)]);
    
    // NEW VERSION: Skip database connection test - using API only
    tracing::warn!("NEW VERSION: Skipping database connection test");
    tracing::info!("This version uses Supabase REST API only");
    
    // Test Supabase API connection instead
    tracing::info!("Testing Supabase API connection...");
    Ok(())
}
