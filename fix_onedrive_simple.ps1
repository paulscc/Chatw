Write-Host "Solucionando conflicto de OneDrive con Rust..." -ForegroundColor Green

# Excluir target de .gitignore
if (-not (Test-Path ".gitignore")) {
    "# Rust ignore files" | Out-File -FilePath ".gitignore"
}
$gitignoreContent = Get-Content ".gitignore" -ErrorAction SilentlyContinue
if ($gitignoreContent -notcontains "target/") {
    Add-Content ".gitignore" "`n# Rust`ntarget/`ntarget/debug/`ntarget/release/"
}

# Detener procesos
Stop-Process -Name "cargo", "rustc", "proyect-chat" -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

# Limpiar y compilar
Write-Host "Limpiando y compilando..." -ForegroundColor Yellow
if (Test-Path "target") {
    Remove-Item -Path "target" -Recurse -Force -ErrorAction SilentlyContinue
}
cargo clean
Start-Sleep -Seconds 3

# Compilar
Write-Host "Compilando proyecto..." -ForegroundColor Yellow
cargo build --bin proyect-chat

# Ejecutar
Write-Host "Iniciando backend en http://127.0.0.1:8080..." -ForegroundColor Green
cargo run --bin proyect-chat
