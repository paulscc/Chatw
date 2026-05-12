# ProyectChat Backend

Backend en Rust para la aplicación de chat empresarial ProyectChat.

## Tecnologías

- **Rust** - Lenguaje principal
- **Actix-web** - Framework web
- **SQLx** - Cliente de base de datos
- **PostgreSQL (Supabase)** - Base de datos
- **Redis (Upstash)** - Caché y mensajería
- **WebSockets** - Comunicación en tiempo real

## Variables de entorno

Las siguientes variables deben configurarse en Render:

### Obligatorias
- `REDIS_URL` - URL de conexión a Redis de Upstash
- `SUPABASE_URL` - URL de la API de Supabase
- `SUPABASE_ANON_KEY` - Clave anónima de Supabase
- `SUPABASE_SERVICE_ROLE_KEY` - Clave de servicio de Supabase
- `JWT_SECRET` - Secreto para tokens JWT

### Opcionales
- `PORT` - Puerto del servidor (Render asigna automáticamente)
- `SERVER_ADDRESS` - Dirección del servidor (default: 0.0.0.0:8080)

## Despliegue en Render

1. Sube esta carpeta a un repositorio GitHub separado
2. Crea un nuevo servicio Docker en Render
3. Configura las variables de entorno mencionadas arriba
4. Render construirá y desplegará automáticamente

## Endpoints principales

### Health Check
- `GET /api/health` - Verificar estado del servidor

### WebSockets
- `ws://your-app-url/ws/{profile_id}` - Conexión WebSocket

### API REST
- Perfiles, Workspaces, Canales, Mensajes, etc.

## Estructura del proyecto

```
back/
├── src/                 # Código fuente Rust
├── migrations/          # Migraciones SQL
├── static/             # Archivos estáticos
├── Dockerfile          # Configuración Docker
├── Cargo.toml          # Dependencias
└── sqlx-data.json      # Metadatos SQLx
```

## Desarrollo local

```bash
# Instalar dependencias
cargo build

# Ejecutar servidor
cargo run
```

El servidor iniciará en `http://127.0.0.1:8080`
