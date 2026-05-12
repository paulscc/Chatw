# --- ETAPA DE CONSTRUCCIÓN ---
FROM rust:1.75-slim-bookworm as builder

# Instalar dependencias de compilación
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Usar el archivo sqlx-data.json para compilar sin DB activa
ENV SQLX_OFFLINE=true

# Compilar para release
RUN cargo build --release

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
