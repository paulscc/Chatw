use std::env;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Cargar variables de entorno
    dotenv::dotenv().ok();
    
    println!("🔍 Probando conexión directa a base de datos...");
    
    // Obtener URL de la base de datos
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL no está configurada");
    
    println!("📍 URL: {}", database_url);
    
    // Intentar diferentes configuraciones de conexión
    let connection_attempts = vec![
        database_url.clone(),
        // Intentar sin SSL mode
        database_url.replace("?sslmode=require", ""),
        // Intentar con SSL mode prefer
        database_url.replace("?sslmode=require", "?sslmode=prefer"),
    ];
    
    for (i, url) in connection_attempts.iter().enumerate() {
        println!("\n🔄 Intento {}: {}", i + 1, url);
        
        match PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(url)
            .await
        {
            Ok(pool) => {
                println!("✅ Pool creado exitosamente");
                
                // Probar consulta simple
                match sqlx::query("SELECT 1 as test")
                    .fetch_one(&pool)
                    .await
                {
                    Ok(_) => {
                        println!("✅ Consulta de prueba exitosa");
                        println!("🎉 Conexión a base de datos establecida");
                        return Ok(());
                    }
                    Err(e) => {
                        println!("❌ Error en consulta: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Error de conexión: {}", e);
            }
        }
    }
    
    println!("\n❌ Todos los intentos de conexión fallaron");
    Ok(())
}
