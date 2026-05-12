# --- ETAPA DE CONSTRUCCIÓN ---
FROM rust:1.81-bookworm as builder

# Instalar dependencias de compilación mínimas
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Forzar reconstrucción completa invalidando caché
RUN echo "Build timestamp: $(date)" > /tmp/build_info

# Limitar jobs paralelos para reducir uso de RAM
ENV CARGO_BUILD_JOBS=2
ENV CARGO_NET_RETRY=10

# Copiar Cargo.toml primero para cache
COPY Cargo.toml Cargo.lock ./

# Crear dummy src para cache de dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Usar el archivo sqlx-data.json para compilar sin DB activa
ENV SQLX_OFFLINE=true

# Construir dependencias primero
RUN cargo build --release && rm -rf src

# Copiar código fuente real
# Cache bust: force rebuild when source changes
ENV SOURCE_VERSION=2024-05-12-v2
COPY src ./src
COPY migrations ./migrations
COPY sqlx-data.json ./

# Compilar aplicación completa
RUN cargo build --release

# Limpiar para reducir tamaño
RUN rm -rf /app/target/release/deps /app/target/release/build /app/target/release/*.rlib

# --- ETAPA DE EJECUCIÓN ---
FROM debian:bookworm-slim

# Instalar certificados SSL necesarios para conectar con Supabase y Upstash
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/proyect-chat ./backend

# Exponer puerto (Render usa la variable PORT automáticamente)
EXPOSE 8080

# Configurar variables de entorno por defecto
ENV PORT=8080
ENV SERVER_ADDRESS=0.0.0.0:8080

# Iniciar el servidor
CMD ["./backend"]
