Write-Host "Verificando integracion de Supabase..." -ForegroundColor Green

# Verificar archivos
$files = @("src/supabase.rs", "src/auth_supabase.rs", "src/config.rs", "src/error.rs")
foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "OK: $file" -ForegroundColor Green
    } else {
        Write-Host "ERROR: $file" -ForegroundColor Red
    }
}

# Verificar .env
if (Test-Path ".env") {
    $content = Get-Content ".env"
    if ($content -match "SUPABASE_URL" -and $content -match "SUPABASE_ANON_KEY") {
        Write-Host "OK: Configuracion .env" -ForegroundColor Green
    } else {
        Write-Host "ERROR: Configuracion .env" -ForegroundColor Red
    }
}

# Probar conexion API
try {
    $envContent = Get-Content ".env"
    $url = ($envContent | Select-String "SUPABASE_URL").ToString().Split("=")[1]
    $key = ($envContent | Select-String "SUPABASE_SERVICE_ROLE_KEY").ToString().Split("=")[1]
    
    $headers = @{
        "apikey" = $key
        "Authorization" = "Bearer $key"
    }
    
    $response = Invoke-RestMethod -Uri "$url/rest/v1/" -Method Get -Headers $headers
    Write-Host "OK: Conexion API" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Conexion API - $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Verificacion completada" -ForegroundColor Green
