# Script simple para probar conexión a Supabase usando psql si está disponible
# o usando curl para probar la API de Supabase

Write-Host "🔍 Probando conexión a Supabase..." -ForegroundColor Green

# Leer variables de entorno desde .env
$envContent = Get-Content ".env" | Where-Object { $_ -notmatch "^#" -and $_.trim() -ne "" }
foreach ($line in $envContent) {
    if ($line -match "^(.+?)=(.*)$") {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

$dbUrl = $env:DATABASE_URL
Write-Host "📍 URL de base de datos: $dbUrl" -ForegroundColor Yellow

# Extraer host y puerto para prueba de conectividad
if ($dbUrl -match "@([^:]+):(\d+)") {
    $host = $matches[1]
    $port = $matches[2]
    Write-Host "🔌 Probando conectividad de red a $host`:$port..." -ForegroundColor Yellow
    
    try {
        $tcpClient = New-Object System.Net.Sockets.TcpClient
        $tcpClient.Connect($host, [int]$port)
        $tcpClient.Close()
        Write-Host "✅ Conectividad de red exitosa" -ForegroundColor Green
    } catch {
        Write-Host "❌ Error de conectividad de red: $($_.Exception.Message)" -ForegroundColor Red
        exit 1
    }
}

# Probar con psql si está disponible
try {
    $psqlVersion = & psql --version 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "🔧 psql encontrado, probando conexión directa..." -ForegroundColor Yellow
        $result = & psql "$dbUrl" -c "SELECT version() as version, NOW() as current_time;" 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Conexión exitosa con psql:" -ForegroundColor Green
            Write-Host $result
        } else {
            Write-Host "❌ Error con psql: $result" -ForegroundColor Red
        }
    }
} catch {
    Write-Host "ℹ️ psql no encontrado, omitiendo prueba directa" -ForegroundColor Yellow
}

# Probar con Test-NetConnection como alternativa
Write-Host "🔌 Probando conexión TCP con Test-NetConnection..." -ForegroundColor Yellow
$testResult = Test-NetConnection -ComputerName $host -Port $port -InformationLevel Detailed
if ($testResult.TcpTestSucceeded) {
    Write-Host "✅ Conexión TCP exitosa" -ForegroundColor Green
    Write-Host "📊 Detalles: Latencia=$($testResult.PingReplyDetails.RoundtripTime)ms" -ForegroundColor Cyan
} else {
    Write-Host "❌ Conexión TCP fallida" -ForegroundColor Red
}

Write-Host "🎉 Prueba de conexión completada" -ForegroundColor Green
