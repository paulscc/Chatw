use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Cargar variables de entorno
    dotenv::dotenv().ok();
    
    // Obtener URL de la base de datos
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL no está configurada");
    
    println!("🔍 Probando conexión a Supabase...");
    println!("📍 URL: {}", database_url);
    
    // Crear pool de conexiones
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await;
    
    match pool {
        Ok(pool) => {
            println!("✅ Pool de conexiones creado exitosamente");
            
            // Probar conexión con una consulta simple
            match sqlx::query("SELECT version() as version, NOW() as current_time")
                .fetch_one(&pool)
                .await
            {
                Ok(row) => {
                    let version: String = row.get("version");
                    let current_time: chrono::NaiveDateTime = row.get("current_time");
                    
                    println!("✅ Conexión exitosa a Supabase!");
                    println!("📊 Versión de PostgreSQL: {}", version);
                    println!("⏰ Hora actual del servidor: {}", current_time);
                    
                    // Probar consulta a tablas (si existen)
                    match sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' LIMIT 5")
                        .fetch_all(&pool)
                        .await
                    {
                        Ok(tables) => {
                            println!("📋 Tablas encontradas en el esquema público:");
                            for table in tables {
                                let table_name: String = table.get("table_name");
                                println!("   - {}", table_name);
                            }
                        }
                        Err(_) => {
                            println!("ℹ️  No se encontraron tablas o no hay permiso para acceder a information_schema");
                        }
                    }
                    
                    println!("🎉 Prueba de conexión completada exitosamente");
                }
                Err(e) => {
                    println!("❌ Error al ejecutar consulta de prueba: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            println!("❌ Error al conectar a Supabase: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
