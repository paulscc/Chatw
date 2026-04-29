use crate::db::Database;
use crate::config::Config;

pub async fn test_supabase_connection() -> anyhow::Result<()> {
    println!("🔍 Iniciando prueba de conexión a Supabase...");
    
    // Cargar configuración desde variables de entorno
    let config = Config::from_env()?;
    println!("📍 URL de base de datos: {}", &config.database_url[..50]);
    println!("📍 URL completa: {}", config.database_url);
    
    // Intentar conectar
    match Database::new(&config.database_url).await {
        Ok(db) => {
            println!("✅ Conexión exitosa a Supabase!");
            
            // Probar consulta simple
            match sqlx::query("SELECT version() as version, NOW() as current_time")
                .fetch_one(&db.pool)
                .await
            {
                Ok(row) => {
                    let version: String = row.get("version");
                    let current_time: chrono::NaiveDateTime = row.get("current_time");
                    
                    println!("📊 Versión de PostgreSQL: {}", version);
                    println!("⏰ Hora actual del servidor: {}", current_time);
                    
                    // Listar tablas si existen
                    match sqlx::query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' LIMIT 10")
                        .fetch_all(&db.pool)
                        .await
                    {
                        Ok(tables) => {
                            println!("📋 Tablas encontradas:");
                            for table in tables {
                                let table_name: String = table.get("table_name");
                                println!("   - {}", table_name);
                            }
                        }
                        Err(e) => {
                            println!("ℹ️  No se pudieron listar tablas: {}", e);
                        }
                    }
                    
                    println!("🎉 Prueba de conexión completada exitosamente");
                }
                Err(e) => {
                    println!("❌ Error en consulta de prueba: {}", e);
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            println!("❌ Error al conectar a Supabase: {}", e);
            return Err(e);
        }
    }
    
    Ok(())
}
