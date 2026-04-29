# Script para solucionar el problema de OneDrive con compilación de Rust
Write-Host "🔧 Solucionando conflicto de OneDrive con Rust..." -ForegroundColor Green

# Paso 1: Excluir directorio target de OneDrive
Write-Host "📁 Excluyendo directorio target de OneDrive..." -ForegroundColor Yellow

# Crear archivo .gitignore si no existe
if (-not (Test-Path ".gitignore")) {
    "# Rust ignore files" | Out-File -FilePath ".gitignore"
    Write-Host "✅ .gitignore creado" -ForegroundColor Green
}

# Agregar target a .gitignore si no está
$gitignoreContent = Get-Content ".gitignore" -ErrorAction SilentlyContinue
if ($gitignoreContent -notcontains "target/") {
    Add-Content ".gitignore" "`n# Rust`ntarget/`ntarget/debug/`ntarget/release/"
    Write-Host "✅ Directorio target agregado a .gitignore" -ForegroundColor Green
} else {
    Write-Host "✅ target/ ya está en .gitignore" -ForegroundColor Green
}

# Crear archivo de exclusión para OneDrive
$oneDriveExcludeContent = @"
# OneDrive exclusions for Rust project
target/
target/debug/
target/release/
*.rcgu.o
*.pdb
*.ilk
*.exp
*.lib
"@

$excludeFile = ".onedriveignore"
if (-not (Test-Path $excludeFile)) {
    $oneDriveExcludeContent | Out-File -FilePath $excludeFile
    Write-Host "✅ Archivo de exclusión de OneDrive creado" -ForegroundColor Green
} else {
    Write-Host "✅ Archivo de exclusión de OneDrive ya existe" -ForegroundColor Green
}

# Paso 2: Detener procesos de Rust que puedan estar bloqueando
Write-Host "🔄 Deteniendo procesos de Rust..." -ForegroundColor Yellow
Stop-Process -Name "cargo", "rustc", "proyect-chat" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Paso 3: Limpiar compilaciones anteriores
Write-Host "🧹 Limpiando compilaciones anteriores..." -ForegroundColor Yellow
if (Test-Path "target") {
    Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "✅ Directorio target eliminado" -ForegroundColor Green
}

# Paso 4: Limpiar con cargo clean
Write-Host "🧽 Ejecutando cargo clean..." -ForegroundColor Yellow
cargo clean
Write-Host "✅ Cargo clean completado" -ForegroundColor Green

# Esperar a que OneDrive termine de sincronizar
Write-Host "⏳ Esperando a que OneDrive termine de sincronizar..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

# Paso 5: Compilar el proyecto
Write-Host "🔨 Compilando el proyecto..." -ForegroundColor Yellow
Write-Host "Comando: cargo build --bin proyect-chat" -ForegroundColor Cyan

try {
    $buildResult = cargo build --bin proyect-chat
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Compilación exitosa" -ForegroundColor Green
    } else {
        Write-Host "❌ Error en compilación" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "❌ Error durante compilación: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Paso 6: Ejecutar el backend
Write-Host "🚀 Iniciando el backend..." -ForegroundColor Green
Write-Host "Servidor iniciará en: http://127.0.0.1:8080" -ForegroundColor Cyan
Write-Host "Presiona Ctrl+C para detener el servidor" -ForegroundColor Yellow
Write-Host ""

try {
    cargo run --bin proyect-chat
} catch {
    Write-Host "❌ Error al ejecutar el backend: $($_.Exception.Message)" -ForegroundColor Red
    
    # Intentar ejecutar el binario directamente
    $executablePath = "target\debug\proyect-chat.exe"
    if (Test-Path $executablePath) {
        Write-Host "🔄 Intentando ejecutar directamente..." -ForegroundColor Yellow
        & $executablePath
    } else {
        Write-Host "❌ No se encontró el ejecutable" -ForegroundColor Red
    }
}

Write-Host "🏁 Proceso completado" -ForegroundColor Green
