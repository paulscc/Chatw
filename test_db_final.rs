use std::env;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    println!("🔍 Prueba final de conexión a base de datos Supabase");
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL no configurada");
    
    println!("📍 URL: {}", database_url);
    
    // Configuración optimizada para Supabase
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&database_url)
        .await;
    
    match pool {
        Ok(pool) => {
            println!("✅ Pool de conexiones creado");
            
            // Prueba básica
            match sqlx::query("SELECT version(), NOW()")
                .fetch_one(&pool)
                .await
            {
                Ok(row) => {
                    let version: String = row.get(0);
                    let time: chrono::NaiveDateTime = row.get(1);
                    println!("✅ Conexión exitosa");
                    println!("📊 PostgreSQL: {}", version);
                    println!("⏰ Hora servidor: {}", time);
                }
                Err(e) => {
                    println!("❌ Error consulta: {}", e);
                    return Err(e.into());
                }
            }
            
            // Listar tablas
            match sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'")
                .fetch_all(&pool)
                .await
            {
                Ok(tables) => {
                    println!("📋 Tablas encontradas:");
                    for table in tables {
                        let name: String = table.get(0);
                        println!("   - {}", name);
                    }
                }
                Err(e) => {
                    println!("ℹ️ No se pudieron listar tablas: {}", e);
                }
            }
            
            println!("🎉 Conexión a Supabase establecida correctamente");
        }
        Err(e) => {
            println!("❌ Error conexión: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}
