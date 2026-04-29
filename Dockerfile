FROM rust:latest as builder

WORKDIR /app

# Copy Cargo files first to leverage Docker layer caching
COPY Cargo.toml ./

# Create a dummy main.rs to build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --quiet
RUN rm -rf src

# Copy the actual source code
COPY src ./src
COPY static ./static

# Build the actual application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/proyect-chat /app/proyect-chat

# Copy static files
COPY static ./static

# Expose port
EXPOSE 8080

# Run the application
CMD ["./proyect-chat"]
