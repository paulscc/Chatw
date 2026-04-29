Write-Host "Iniciando backend del proyecto chat..." -ForegroundColor Green

# Verificar si Rust y Cargo están instalados
try {
    $rustVersion = rustc --version
    $cargoVersion = cargo --version
    Write-Host "Rust: $rustVersion" -ForegroundColor Cyan
    Write-Host "Cargo: $cargoVersion" -ForegroundColor Cyan
} catch {
    Write-Host "ERROR: Rust/Cargo no está instalado" -ForegroundColor Red
    Write-Host "Por favor instala Rust desde: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Verificar archivo Cargo.toml
if (Test-Path "Cargo.toml") {
    Write-Host "OK: Cargo.toml encontrado" -ForegroundColor Green
} else {
    Write-Host "ERROR: Cargo.toml no encontrado" -ForegroundColor Red
    exit 1
}

# Verificar archivo .env
if (Test-Path ".env") {
    Write-Host "OK: .env encontrado" -ForegroundColor Green
} else {
    Write-Host "WARNING: .env no encontrado, usando valores por defecto" -ForegroundColor Yellow
}

# Limpiar compilaciones anteriores si hay problemas
Write-Host "Limpiando compilaciones anteriores..." -ForegroundColor Yellow
cargo clean 2>$null

# Compilar y ejecutar
Write-Host "Compilando y ejecutando el backend..." -ForegroundColor Yellow
Write-Host "Comando: cargo run --bin proyect-chat" -ForegroundColor Cyan
Write-Host "El servidor iniciará en: http://127.0.0.1:8080" -ForegroundColor Green
Write-Host "Presiona Ctrl+C para detener el servidor" -ForegroundColor Yellow
Write-Host ""

try {
    cargo run --bin proyect-chat
} catch {
    Write-Host "ERROR: No se pudo ejecutar el backend" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    
    # Intentar alternativas
    Write-Host "Intentando alternativas..." -ForegroundColor Yellow
    
    # Opción 1: Solo compilar
    Write-Host "1. Intentando compilar solo..." -ForegroundColor Yellow
    cargo build --bin proyect-chat
    
    # Opción 2: Ejecutar directamente
    if (Test-Path "target\debug\proyect-chat.exe") {
        Write-Host "2. Ejecutable encontrado, iniciando..." -ForegroundColor Yellow
        & ".\target\debug\proyect-chat.exe"
    } else {
        Write-Host "ERROR: No se encontró el ejecutable" -ForegroundColor Red
    }
}

Write-Host "Backend detenido" -ForegroundColor Green
