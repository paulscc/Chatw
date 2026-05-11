Write-Host "Probando conexión directa a Supabase..." -ForegroundColor Green

$connectionString = "postgresql://postgres:i3eJVBDY1AOqgLNh@db.peqlqnzkthdrlisvxwgh.supabase.co:6543/postgres?sslmode=require"

Write-Host "Intentando conectar con: $connectionString" -ForegroundColor Cyan

# Intentar usar psql si está disponible
try {
    $result = psql "$connectionString" -c "SELECT version();" -t -A 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Conexión exitosa a Supabase" -ForegroundColor Green
        Write-Host "Versión: $result" -ForegroundColor Yellow
    } else {
        Write-Host "❌ Error de conexión: $result" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Error al ejecutar psql: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "psql no está disponible o no se puede conectar" -ForegroundColor Yellow
}

# Intentar con Test-NetConnection para verificar conectividad de red
Write-Host "Verificando conectividad de red..." -ForegroundColor Yellow
try {
    $netTest = Test-NetConnection -ComputerName "db.bsmafajdvzbpxtepxysz.supabase.co" -Port 5432 -WarningAction SilentlyContinue
    if ($netTest.TcpTestSucceeded) {
        Write-Host "✅ Conectividad de red OK al puerto 5432" -ForegroundColor Green
    } else {
        Write-Host "❌ No hay conectividad de red al puerto 5432" -ForegroundColor Red
        Write-Host "Source address: $($netTest.SourceAddress)" -ForegroundColor Yellow
        Write-Host "Remote address: $($netTest.RemoteAddress)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "❌ Error en prueba de red: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba completada" -ForegroundColor Green
