# Script para probar API de Supabase con claves JWT correctas
Write-Host "Probando API de Supabase con claves JWT..." -ForegroundColor Green

$supabaseUrl = "https://bsmafajdvzbpxtepxysz.supabase.co"
$anonKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE3Nzc0NTg2MjgsImV4cCI6MjA5MzAzNDYyOH0.qI-E3efN75xEX3fdhvGoi84k0_aOOo2u9hMnxe2lbJM"
$serviceKey = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImJzbWFmYWpkdnpicHh0ZXB4eXN6Iiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTc3NzQ1ODYyOCwiZXhwIjoyMDkzMDM0NjI4fQ.hGsfYrvF__FtSAfHMh1e0B_jgAOrYR40TBsoJoMUgzo"

Write-Host "URL: $supabaseUrl" -ForegroundColor Yellow

# Probar con clave anonima
try {
    $headers = @{
        "apikey" = $anonKey
        "Authorization" = "Bearer $anonKey"
        "Content-Type" = "application/json"
    }
    
    Write-Host "Probando con clave anonima..." -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Conexion con clave anonima exitosa" -ForegroundColor Green
    Write-Host "Respuesta: $response" -ForegroundColor Cyan
    
} catch {
    Write-Host "Error con clave anonima: $($_.Exception.Message)" -ForegroundColor Red
}

# Probar con clave de servicio
try {
    $headers = @{
        "apikey" = $serviceKey
        "Authorization" = "Bearer $serviceKey"
        "Content-Type" = "application/json"
    }
    
    Write-Host "Probando con clave de servicio..." -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Conexion con clave de servicio exitosa" -ForegroundColor Green
    Write-Host "Respuesta: $response" -ForegroundColor Cyan
    
} catch {
    Write-Host "Error con clave de servicio: $($_.Exception.Message)" -ForegroundColor Red
}

# Probar obtener informacion del proyecto
try {
    $headers = @{
        "apikey" = $serviceKey
        "Authorization" = "Bearer $serviceKey"
    }
    
    Write-Host "Probando obtener informacion del proyecto..." -ForegroundColor Yellow
    $response = Invoke-RestMethod -Uri "$supabaseUrl/rest/v1/" -Method Get -Headers $headers -ErrorAction Stop
    Write-Host "Informacion del proyecto obtenida" -ForegroundColor Green
    
} catch {
    Write-Host "Error obteniendo informacion: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "Prueba de API completada" -ForegroundColor Green
